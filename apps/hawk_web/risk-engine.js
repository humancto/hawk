// =============================================================================
// Hawk Risk Assessment Engine
// =============================================================================
// Pure JavaScript, zero dependencies. Works with hawk graph data shape:
//   nodes: [{ id, kind, name, props: { runtime, timeout, memory_size, ... } }]
//   edges: [{ from, to, kind }]
//
// Usage:
//   const engine = HawkRiskEngine(nodes, edges);
//   const scores  = engine.computeAllRiskScores();
//   const aps     = engine.findArticulationPoints();
//   const path    = engine.longestCriticalPath();
//   const blast   = engine.blastRadius('some-node-id');
//   const clusters = engine.clusterHealth();
//   const whatif  = engine.whatIfRemove('some-node-id');
//   const health  = engine.graphHealthScore();
//   const recs    = engine.recommendations('some-node-id');
// =============================================================================

"use strict";

function HawkRiskEngine(nodes, edges) {
  // =========================================================================
  // 0. GRAPH PREPROCESSING — Build adjacency lists once, reuse everywhere
  // =========================================================================

  const nodeMap = new Map(); // id -> node object
  const adjOut = new Map(); // id -> [{ to, kind }]
  const adjIn = new Map(); // id -> [{ from, kind }]
  const nodeIds = []; // ordered list of all node ids

  for (let i = 0; i < nodes.length; i++) {
    const n = nodes[i];
    nodeMap.set(n.id, n);
    adjOut.set(n.id, []);
    adjIn.set(n.id, []);
    nodeIds.push(n.id);
  }

  for (let i = 0; i < edges.length; i++) {
    const e = edges[i];
    // Guard against edges referencing nodes not in the graph
    if (!nodeMap.has(e.from) || !nodeMap.has(e.to)) continue;
    adjOut.get(e.from).push({ to: e.to, kind: e.kind });
    adjIn.get(e.to).push({ from: e.from, kind: e.kind });
  }

  // Undirected adjacency for articulation point detection
  const adjUndirected = new Map();
  for (const id of nodeIds) {
    adjUndirected.set(id, new Set());
  }
  for (let i = 0; i < edges.length; i++) {
    const e = edges[i];
    if (!nodeMap.has(e.from) || !nodeMap.has(e.to)) continue;
    adjUndirected.get(e.from).add(e.to);
    adjUndirected.get(e.to).add(e.from);
  }

  // =========================================================================
  // 1. RISK SCORE ENGINE (0–100 per node)
  // =========================================================================
  //
  // Signals (each produces a value in [0, 1]):
  //   fan_in       (weight 25) — high fan-in = single point of failure
  //   fan_out      (weight 15) — high fan-out = large blast radius
  //   bridge       (weight 20) — articulation point that disconnects the graph
  //   runtime_age  (weight 10) — deprecated/EOL runtimes
  //   timeout_risk (weight 10) — timeout >= 300s or no timeout set
  //   isolated     (weight  5) — zero connections
  //   single_conn  (weight  5) — exactly one edge in or out
  //   missing_dlq  (weight 10) — no dead-letter queue / error handling visible
  //
  // Total weights = 100. Final score = sum(signal_i * weight_i).

  const RISK_WEIGHTS = {
    fan_in: 25,
    fan_out: 15,
    bridge: 20,
    runtime_age: 10,
    timeout_risk: 10,
    isolated: 5,
    single_conn: 5,
    missing_dlq: 10,
  };

  // Deprecated / EOL runtimes as of 2026
  const DEPRECATED_RUNTIMES = new Set([
    "nodejs10.x",
    "nodejs12.x",
    "nodejs14.x",
    "nodejs16.x",
    "python2.7",
    "python3.6",
    "python3.7",
    "python3.8",
    "dotnetcore2.1",
    "dotnetcore3.1",
    "dotnet5.0",
    "dotnet6",
    "java8",
    "java8.al2",
    "ruby2.5",
    "ruby2.7",
    "go1.x",
    "provided",
  ]);

  // Nearing EOL — partial risk
  const AGING_RUNTIMES = new Set([
    "nodejs18.x",
    "python3.9",
    "python3.10",
    "java11",
    "dotnet7",
    "ruby3.2",
  ]);

  // Precompute max fan-in / fan-out across all nodes for normalization
  let maxFanIn = 1,
    maxFanOut = 1;
  for (const id of nodeIds) {
    const fi = adjIn.get(id).length;
    const fo = adjOut.get(id).length;
    if (fi > maxFanIn) maxFanIn = fi;
    if (fo > maxFanOut) maxFanOut = fo;
  }

  // Cache for articulation points (computed lazily, once)
  let _articulationPoints = null;

  function _getArticulationPoints() {
    if (_articulationPoints === null) {
      _articulationPoints = findArticulationPoints();
    }
    return _articulationPoints;
  }

  /**
   * Compute the risk signal breakdown for a single node.
   * Returns an object { signals: { name: 0-1 }, score: 0-100 }.
   */
  function computeNodeRisk(nodeId) {
    const node = nodeMap.get(nodeId);
    if (!node) return { signals: {}, score: 0 };

    const fanIn = adjIn.get(nodeId).length;
    const fanOut = adjOut.get(nodeId).length;
    const totalDeg = fanIn + fanOut;
    const props = node.props || {};
    const aps = _getArticulationPoints();

    const signals = {};

    // --- Fan-in signal ---
    // Normalize with a soft ceiling: score = min(1, fanIn / max(5, maxFanIn * 0.6))
    // This means even modest fan-in relative to the graph scores meaningfully
    const fanInThreshold = Math.max(5, maxFanIn * 0.6);
    signals.fan_in = Math.min(1, fanIn / fanInThreshold);

    // --- Fan-out signal ---
    const fanOutThreshold = Math.max(4, maxFanOut * 0.6);
    signals.fan_out = Math.min(1, fanOut / fanOutThreshold);

    // --- Bridge node (articulation point) ---
    signals.bridge = aps.has(nodeId) ? 1.0 : 0.0;

    // --- Runtime age ---
    const runtime = (props.runtime || "").toLowerCase();
    if (DEPRECATED_RUNTIMES.has(runtime)) {
      signals.runtime_age = 1.0;
    } else if (AGING_RUNTIMES.has(runtime)) {
      signals.runtime_age = 0.5;
    } else if (runtime === "" && node.kind === "Lambda") {
      // Lambda without runtime info — suspicious
      signals.runtime_age = 0.3;
    } else {
      signals.runtime_age = 0.0;
    }

    // --- Timeout risk ---
    if (node.kind === "Lambda" || node.kind === "StepFunction") {
      const timeout = props.timeout;
      if (timeout === undefined || timeout === null) {
        signals.timeout_risk = 0.7; // No timeout configured — risky
      } else if (timeout >= 300) {
        signals.timeout_risk = Math.min(1.0, timeout / 900);
      } else if (timeout >= 120) {
        signals.timeout_risk = 0.3;
      } else {
        signals.timeout_risk = 0.0;
      }
    } else {
      signals.timeout_risk = 0.0;
    }

    // --- Isolated node ---
    signals.isolated = totalDeg === 0 ? 1.0 : 0.0;

    // --- Single consumer/producer ---
    if (totalDeg === 1) {
      signals.single_conn = 1.0;
    } else if (totalDeg === 2 && (fanIn === 0 || fanOut === 0)) {
      // All connections in one direction only
      signals.single_conn = 0.5;
    } else {
      signals.single_conn = 0.0;
    }

    // --- Missing DLQ ---
    // Heuristic: SQS queues and Lambdas with event source mappings should have DLQ
    // We check if the node name contains "dlq" or if any connected node is a DLQ
    if (node.kind === "SqsQueue") {
      const name = (node.name || "").toLowerCase();
      if (
        name.includes("dlq") ||
        name.includes("dead-letter") ||
        name.includes("deadletter")
      ) {
        signals.missing_dlq = 0.0; // This IS a DLQ
      } else {
        // Check if this queue has a sibling DLQ connected
        const hasDlqPeer = _hasConnectedDlq(nodeId);
        signals.missing_dlq = hasDlqPeer ? 0.0 : 0.8;
      }
    } else if (node.kind === "Lambda") {
      // Check if lambda has an event source mapping from SQS (via incoming edges)
      const hasSqsTrigger = adjIn.get(nodeId).some((e) => {
        const srcNode = nodeMap.get(e.from);
        return srcNode && srcNode.kind === "SqsQueue";
      });
      if (hasSqsTrigger) {
        // Lambda triggered by SQS should ideally have error handling
        const hasErrorHandling = _hasConnectedDlq(nodeId);
        signals.missing_dlq = hasErrorHandling ? 0.0 : 0.6;
      } else if (node.kind === "Lambda" && adjIn.get(nodeId).length > 0) {
        // Any triggered lambda benefits from error handling
        signals.missing_dlq = 0.3;
      } else {
        signals.missing_dlq = 0.0;
      }
    } else if (node.kind === "SnsTopic") {
      signals.missing_dlq = 0.4; // SNS topics should have DLQ on subscriptions
    } else {
      signals.missing_dlq = 0.0;
    }

    // --- Compute weighted sum ---
    let score = 0;
    for (const key in RISK_WEIGHTS) {
      score += (signals[key] || 0) * RISK_WEIGHTS[key];
    }
    score = Math.round(Math.min(100, Math.max(0, score)));

    return { signals, score, tier: riskTier(score) };
  }

  /**
   * Check if a node has a connected DLQ (by name heuristic).
   */
  function _hasConnectedDlq(nodeId) {
    // Check 1-hop and 2-hop neighbors for a DLQ node
    const checked = new Set([nodeId]);
    const queue = [nodeId];
    for (let depth = 0; depth < 2; depth++) {
      const nextQueue = [];
      for (const cid of queue) {
        const neighbors = [];
        for (const e of adjOut.get(cid) || []) neighbors.push(e.to);
        for (const e of adjIn.get(cid) || []) neighbors.push(e.from);
        for (const nid of neighbors) {
          if (checked.has(nid)) continue;
          checked.add(nid);
          const nn = nodeMap.get(nid);
          if (nn) {
            const name = (nn.name || "").toLowerCase();
            if (
              name.includes("dlq") ||
              name.includes("dead-letter") ||
              name.includes("deadletter")
            ) {
              return true;
            }
          }
          nextQueue.push(nid);
        }
      }
      queue.length = 0;
      for (const x of nextQueue) queue.push(x);
    }
    return false;
  }

  /**
   * Compute risk scores for ALL nodes. Returns Map<nodeId, {signals, score}>.
   */
  function computeAllRiskScores() {
    const results = new Map();
    for (const id of nodeIds) {
      results.set(id, computeNodeRisk(id));
    }
    return results;
  }

  /**
   * Classify a score into a severity tier.
   */
  function riskTier(score) {
    if (score >= 70) return "critical";
    if (score >= 45) return "high";
    if (score >= 20) return "medium";
    return "low";
  }

  // =========================================================================
  // 2. ARTICULATION POINT DETECTION (Tarjan's Algorithm)
  // =========================================================================
  //
  // Finds nodes whose removal disconnects the undirected version of the graph.
  // These are single points of failure in the infrastructure.
  // Returns Set<nodeId>.
  //
  // Time: O(V + E), Space: O(V)

  function findArticulationPoints() {
    const disc = new Map(); // discovery time
    const low = new Map(); // lowest reachable discovery time
    const parent = new Map(); // parent in DFS tree
    const ap = new Set(); // result set
    let timer = 0;

    function dfs(u) {
      disc.set(u, timer);
      low.set(u, timer);
      timer++;
      let childCount = 0;

      for (const v of adjUndirected.get(u)) {
        if (!disc.has(v)) {
          childCount++;
          parent.set(v, u);
          dfs(v);

          // Update low-link
          const lowV = low.get(v);
          const lowU = low.get(u);
          if (lowV < lowU) low.set(u, lowV);

          // u is an articulation point if:
          // (1) u is root of DFS tree and has 2+ children
          if (!parent.has(u) && childCount > 1) {
            ap.add(u);
          }
          // (2) u is not root and low[v] >= disc[u]
          if (parent.has(u) && low.get(v) >= disc.get(u)) {
            ap.add(u);
          }
        } else if (v !== parent.get(u)) {
          // Back edge — update low-link
          const discV = disc.get(v);
          const lowU = low.get(u);
          if (discV < lowU) low.set(u, discV);
        }
      }
    }

    // Run DFS from every unvisited node (handles disconnected components)
    for (const id of nodeIds) {
      if (!disc.has(id)) {
        dfs(id);
      }
    }

    return ap;
  }

  // =========================================================================
  // 3. LONGEST CRITICAL PATH
  // =========================================================================
  //
  // Finds the longest chain of directed trigger/invoke connections.
  // Uses BFS from every source node (in-degree 0). Tracks the longest path
  // discovered to any node. Then backtraces to reconstruct the path.
  //
  // For DAGs this is the classic longest-path in O(V+E).
  // For graphs with cycles, we use a visited guard to avoid infinite loops.

  function longestCriticalPath() {
    // Topological-order based longest path
    // First, compute in-degree for topological sort
    const inDegree = new Map();
    for (const id of nodeIds) inDegree.set(id, 0);
    for (const id of nodeIds) {
      for (const e of adjOut.get(id)) {
        inDegree.set(e.to, (inDegree.get(e.to) || 0) + 1);
      }
    }

    // Kahn's algorithm for topological sort
    const topoOrder = [];
    const queue = [];
    for (const id of nodeIds) {
      if (inDegree.get(id) === 0) queue.push(id);
    }

    const tempInDeg = new Map(inDegree);
    while (queue.length > 0) {
      const u = queue.shift();
      topoOrder.push(u);
      for (const e of adjOut.get(u)) {
        const newDeg = tempInDeg.get(e.to) - 1;
        tempInDeg.set(e.to, newDeg);
        if (newDeg === 0) queue.push(e.to);
      }
    }

    // If there are cycles, some nodes won't be in topoOrder.
    // Add remaining nodes to handle cycles gracefully.
    const inTopo = new Set(topoOrder);
    for (const id of nodeIds) {
      if (!inTopo.has(id)) topoOrder.push(id);
    }

    // Longest path via dynamic programming on topological order
    const dist = new Map(); // id -> longest distance to reach this node
    const pred = new Map(); // id -> predecessor on longest path
    for (const id of nodeIds) {
      dist.set(id, 0);
      pred.set(id, null);
    }

    for (const u of topoOrder) {
      const du = dist.get(u);
      for (const e of adjOut.get(u)) {
        const v = e.to;
        if (du + 1 > dist.get(v)) {
          dist.set(v, du + 1);
          pred.set(v, u);
        }
      }
    }

    // Find the node with the maximum distance
    let maxDist = 0,
      endNode = null;
    for (const id of nodeIds) {
      if (dist.get(id) > maxDist) {
        maxDist = dist.get(id);
        endNode = id;
      }
    }

    if (endNode === null || maxDist === 0) {
      return { path: [], length: 0 };
    }

    // Backtrace to reconstruct the path
    const path = [];
    let cur = endNode;
    while (cur !== null) {
      path.unshift(cur);
      cur = pred.get(cur);
    }

    return {
      path: path,
      length: path.length,
      depth: maxDist,
      nodes: path.map((id) => ({
        id: id,
        name: nodeMap.get(id).name,
        kind: nodeMap.get(id).kind,
      })),
    };
  }

  // =========================================================================
  // 4. BLAST RADIUS CALCULATION
  // =========================================================================
  //
  // Given a node ID, compute all downstream nodes reachable via directed edges.
  // Returns the set of affected nodes with their depth from the failing node.
  //
  // BFS on directed out-edges. O(V + E).

  function blastRadius(nodeId) {
    if (!nodeMap.has(nodeId)) {
      return { origin: nodeId, affected: [], totalCount: 0, maxDepth: 0 };
    }

    const visited = new Map(); // id -> depth
    const queue = [[nodeId, 0]];
    visited.set(nodeId, 0);

    while (queue.length > 0) {
      const [current, depth] = queue.shift();
      for (const e of adjOut.get(current)) {
        if (!visited.has(e.to)) {
          visited.set(e.to, depth + 1);
          queue.push([e.to, depth + 1]);
        }
      }
    }

    // Remove the origin node itself from the affected set
    visited.delete(nodeId);

    const affected = [];
    let maxDepth = 0;
    for (const [id, depth] of visited) {
      const n = nodeMap.get(id);
      affected.push({
        id: id,
        name: n.name,
        kind: n.kind,
        depth: depth,
      });
      if (depth > maxDepth) maxDepth = depth;
    }

    // Sort by depth, then name
    affected.sort((a, b) => a.depth - b.depth || a.name.localeCompare(b.name));

    return {
      origin: nodeId,
      originName: nodeMap.get(nodeId).name,
      affected: affected,
      totalCount: affected.length,
      maxDepth: maxDepth,
      byDepth: _groupByDepth(affected),
    };
  }

  function _groupByDepth(affected) {
    const groups = {};
    for (const a of affected) {
      if (!groups[a.depth]) groups[a.depth] = [];
      groups[a.depth].push(a);
    }
    return groups;
  }

  // =========================================================================
  // 5. CLUSTER HEALTH AGGREGATION
  // =========================================================================
  //
  // Groups nodes by service type (kind) and optionally by name prefix.
  // Computes average, max, and min risk score per cluster.

  function clusterHealth(options) {
    options = options || {};
    const groupBy = options.groupBy || "kind"; // 'kind' | 'prefix'
    const prefixLen = options.prefixLen || 0; // auto-detect if 0

    const allScores = computeAllRiskScores();
    const clusters = new Map(); // groupKey -> { nodes: [], scores: [] }

    for (const id of nodeIds) {
      const node = nodeMap.get(id);
      const risk = allScores.get(id);
      let groupKey;

      if (groupBy === "prefix") {
        groupKey = _extractPrefix(node.name, prefixLen);
      } else {
        groupKey = node.kind;
      }

      if (!clusters.has(groupKey)) {
        clusters.set(groupKey, { key: groupKey, nodes: [], scores: [] });
      }
      const cl = clusters.get(groupKey);
      cl.nodes.push({
        id: id,
        name: node.name,
        kind: node.kind,
        score: risk.score,
      });
      cl.scores.push(risk.score);
    }

    // Compute aggregate stats per cluster
    const results = [];
    for (const [key, cl] of clusters) {
      const scores = cl.scores;
      const sum = scores.reduce((a, b) => a + b, 0);
      const avg = scores.length > 0 ? sum / scores.length : 0;
      const max = scores.length > 0 ? Math.max(...scores) : 0;
      const min = scores.length > 0 ? Math.min(...scores) : 0;

      const highRiskCount = scores.filter((s) => s >= 45).length;

      results.push({
        cluster: key,
        nodeCount: cl.nodes.length,
        avgScore: Math.round(avg * 10) / 10,
        maxScore: max,
        minScore: min,
        highRiskCount: highRiskCount,
        tier: riskTier(Math.round(avg)),
        nodes: cl.nodes.sort((a, b) => b.score - a.score),
      });
    }

    // Sort clusters by average score descending
    results.sort((a, b) => b.avgScore - a.avgScore);

    return results;
  }

  /**
   * Extract a name prefix for grouping. Splits on common delimiters.
   */
  function _extractPrefix(name, fixedLen) {
    if (fixedLen > 0) return name.substring(0, fixedLen);
    // Auto-detect: split on '-', '_', or camelCase boundary; take first segment
    const parts = name.split(/[-_]/);
    if (parts.length >= 2) return parts[0];
    // camelCase split
    const camel = name.replace(/([a-z])([A-Z])/g, "$1-$2").split("-");
    if (camel.length >= 2) return camel[0].toLowerCase();
    return name;
  }

  // =========================================================================
  // 6. "WHAT IF" SIMULATION
  // =========================================================================
  //
  // Simulates removing a node from the graph.
  // Returns:
  //   - disconnectedNodes: nodes that become unreachable from any source
  //   - orphanedEdges: edges that lose a source or target
  //   - affectedDownstream: count of downstream nodes that lose connectivity
  //   - newComponents: number of connected components after removal
  //   - componentSizes: sizes of each component

  function whatIfRemove(nodeId) {
    if (!nodeMap.has(nodeId)) {
      return { error: "Node not found", nodeId: nodeId };
    }

    const removedNode = nodeMap.get(nodeId);

    // Build reduced graph (all nodes except the removed one)
    const remainingIds = nodeIds.filter((id) => id !== nodeId);
    const remainingSet = new Set(remainingIds);

    // Find orphaned edges (edges connected to the removed node)
    const orphanedEdges = [];
    for (let i = 0; i < edges.length; i++) {
      const e = edges[i];
      if (e.from === nodeId || e.to === nodeId) {
        orphanedEdges.push({
          from: e.from,
          to: e.to,
          kind: e.kind,
          reason: e.from === nodeId ? "source_removed" : "target_removed",
        });
      }
    }

    // Build reduced undirected adjacency
    const reducedAdj = new Map();
    for (const id of remainingIds) reducedAdj.set(id, []);
    for (let i = 0; i < edges.length; i++) {
      const e = edges[i];
      if (e.from === nodeId || e.to === nodeId) continue;
      if (!remainingSet.has(e.from) || !remainingSet.has(e.to)) continue;
      reducedAdj.get(e.from).push(e.to);
      reducedAdj.get(e.to).push(e.from);
    }

    // Count connected components in the reduced graph (undirected)
    const visited = new Set();
    const components = [];

    for (const startId of remainingIds) {
      if (visited.has(startId)) continue;
      const component = [];
      const stack = [startId];
      while (stack.length > 0) {
        const u = stack.pop();
        if (visited.has(u)) continue;
        visited.add(u);
        component.push(u);
        for (const v of reducedAdj.get(u)) {
          if (!visited.has(v)) stack.push(v);
        }
      }
      components.push(component);
    }

    // Count connected components in the ORIGINAL graph
    const origVisited = new Set();
    let origComponentCount = 0;
    for (const startId of nodeIds) {
      if (origVisited.has(startId)) continue;
      origComponentCount++;
      const stack = [startId];
      while (stack.length > 0) {
        const u = stack.pop();
        if (origVisited.has(u)) continue;
        origVisited.add(u);
        for (const v of adjUndirected.get(u)) {
          if (!origVisited.has(v)) stack.push(v);
        }
      }
    }

    // Compute blast radius for comparison
    const blast = blastRadius(nodeId);

    // Find nodes that were reachable before but become disconnected
    // A node is "newly disconnected" if it was in the same component as
    // the removed node but is now in a smaller fragment
    const disconnectedNodes = [];
    if (components.length > origComponentCount) {
      // The removal created new components — find the smaller fragments
      // Sort components by size descending; fragments after the first are "disconnected"
      components.sort((a, b) => b.length - a.length);
      for (let i = 1; i < components.length; i++) {
        // Check if this component existed before the removal
        // by checking if any node in it was connected to the removed node
        const wasConnectedToRemoved = components[i].some((id) => {
          return adjUndirected.get(nodeId).has(id);
        });
        if (wasConnectedToRemoved || components.length > origComponentCount) {
          for (const id of components[i]) {
            const n = nodeMap.get(id);
            disconnectedNodes.push({ id: id, name: n.name, kind: n.kind });
          }
        }
      }
    }

    return {
      removedNode: {
        id: nodeId,
        name: removedNode.name,
        kind: removedNode.kind,
      },
      orphanedEdges: orphanedEdges,
      orphanedEdgeCount: orphanedEdges.length,
      disconnectedNodes: disconnectedNodes,
      disconnectedCount: disconnectedNodes.length,
      affectedDownstream: blast.totalCount,
      originalComponents: origComponentCount,
      newComponents: components.length,
      componentSizes: components.map((c) => c.length).sort((a, b) => b - a),
      severityIncrease: components.length - origComponentCount,
      isArticulationPoint: _getArticulationPoints().has(nodeId),
    };
  }

  // =========================================================================
  // 7. GRAPH HEALTH SCORE (0–100 for the entire graph)
  // =========================================================================
  //
  // Composite score based on:
  //   - % of high-risk nodes (weight 30)
  //   - presence of SPOFs / articulation points (weight 25)
  //   - average path redundancy (weight 25)
  //   - isolated node ratio (weight 20)
  //
  // Higher score = healthier graph.

  function graphHealthScore() {
    if (nodeIds.length === 0) {
      return { score: 100, tier: "low", breakdown: {}, details: {} };
    }

    const allScores = computeAllRiskScores();
    const aps = _getArticulationPoints();

    // 1. High-risk node ratio (inverted: fewer high-risk = better health)
    let highRiskCount = 0;
    let criticalCount = 0;
    let totalRisk = 0;
    for (const [id, r] of allScores) {
      totalRisk += r.score;
      if (r.score >= 45) highRiskCount++;
      if (r.score >= 70) criticalCount++;
    }
    const highRiskRatio = highRiskCount / nodeIds.length;
    const highRiskSignal = 1.0 - Math.min(1.0, highRiskRatio * 2.5);
    // If > 40% are high-risk, signal = 0

    // 2. SPOF presence (articulation points)
    const spofRatio = aps.size / Math.max(1, nodeIds.length);
    const spofSignal = 1.0 - Math.min(1.0, spofRatio * 5);
    // If > 20% are SPOFs, signal = 0

    // 3. Average path redundancy
    // For each node with in-degree > 0, how many alternative paths exist?
    // Simple proxy: average in-degree of non-source nodes (higher = more redundant)
    let redundancySum = 0,
      redundancyCount = 0;
    for (const id of nodeIds) {
      const fi = adjIn.get(id).length;
      if (fi > 0) {
        // Redundancy is higher when a node has multiple inputs
        redundancySum += Math.min(1.0, (fi - 1) / 3);
        redundancyCount++;
      }
    }
    const redundancySignal =
      redundancyCount > 0 ? redundancySum / redundancyCount : 0.5; // If no edges, neutral

    // 4. Isolated node ratio (inverted: fewer isolated = better)
    let isolatedCount = 0;
    for (const id of nodeIds) {
      if (adjIn.get(id).length === 0 && adjOut.get(id).length === 0) {
        isolatedCount++;
      }
    }
    const isolatedRatio = isolatedCount / nodeIds.length;
    const isolatedSignal = 1.0 - Math.min(1.0, isolatedRatio * 3);

    // Weighted combination (weights sum to 100)
    const weights = {
      highRisk: 30,
      spof: 25,
      redundancy: 25,
      isolated: 20,
    };

    const score = Math.round(
      highRiskSignal * weights.highRisk +
        spofSignal * weights.spof +
        redundancySignal * weights.redundancy +
        isolatedSignal * weights.isolated,
    );

    const clampedScore = Math.max(0, Math.min(100, score));

    return {
      score: clampedScore,
      tier: healthTier(clampedScore),
      breakdown: {
        highRiskSignal: Math.round(highRiskSignal * 100) / 100,
        spofSignal: Math.round(spofSignal * 100) / 100,
        redundancySignal: Math.round(redundancySignal * 100) / 100,
        isolatedSignal: Math.round(isolatedSignal * 100) / 100,
      },
      details: {
        totalNodes: nodeIds.length,
        totalEdges: edges.length,
        highRiskNodes: highRiskCount,
        criticalNodes: criticalCount,
        spofCount: aps.size,
        isolatedNodes: isolatedCount,
        avgRiskScore: Math.round((totalRisk / nodeIds.length) * 10) / 10,
        longestPath: longestCriticalPath().length,
      },
    };
  }

  function healthTier(score) {
    if (score >= 80) return "healthy";
    if (score >= 60) return "fair";
    if (score >= 40) return "degraded";
    return "critical";
  }

  // =========================================================================
  // 8. RISK RECOMMENDATIONS GENERATOR
  // =========================================================================
  //
  // For each node, generate actionable recommendations based on its risk
  // signal breakdown. Returns an array of recommendation objects.

  function recommendations(nodeId) {
    const node = nodeMap.get(nodeId);
    if (!node) return [];

    const risk = computeNodeRisk(nodeId);
    const signals = risk.signals;
    const recs = [];
    const props = node.props || {};

    // --- Fan-in recommendations ---
    if (signals.fan_in > 0.6) {
      const fanIn = adjIn.get(nodeId).length;
      recs.push({
        severity: "high",
        category: "single-point-of-failure",
        signal: "fan_in",
        title: "High fan-in creates a single point of failure",
        detail:
          "This node has " +
          fanIn +
          " incoming connections. If it fails, " +
          "all " +
          fanIn +
          " upstream services are affected. Consider adding " +
          "redundancy or breaking this into multiple handlers.",
        actions: [
          "Add a dead-letter queue to capture failed invocations",
          "Consider splitting into multiple functions by upstream source",
          "Implement circuit breaker pattern on upstream callers",
          "Add CloudWatch alarms for error rate and throttling",
        ],
      });
    } else if (signals.fan_in > 0.3) {
      recs.push({
        severity: "medium",
        category: "single-point-of-failure",
        signal: "fan_in",
        title: "Moderate fan-in — monitor for bottlenecks",
        detail:
          "This node has " +
          adjIn.get(nodeId).length +
          " incoming connections. " +
          "Monitor for concurrency limits and throttling.",
        actions: [
          "Set up CloudWatch alarms for concurrent executions",
          "Consider reserved concurrency settings",
        ],
      });
    }

    // --- Fan-out recommendations ---
    if (signals.fan_out > 0.6) {
      const fanOut = adjOut.get(nodeId).length;
      recs.push({
        severity: "high",
        category: "blast-radius",
        signal: "fan_out",
        title: "High fan-out creates a large blast radius",
        detail:
          "This node connects to " +
          fanOut +
          " downstream services. A failure " +
          "here could cascade to all of them.",
        actions: [
          "Implement retry logic with exponential backoff for each downstream call",
          "Use SQS or SNS as a buffer between this node and its targets",
          "Add per-target circuit breakers",
          "Consider an orchestrator (Step Functions) instead of direct invocation",
        ],
      });
    }

    // --- Bridge / Articulation point ---
    if (signals.bridge > 0) {
      recs.push({
        severity: "critical",
        category: "graph-connectivity",
        signal: "bridge",
        title: "Articulation point — removal disconnects the graph",
        detail:
          "This node is a bridge in the infrastructure graph. Removing or " +
          "disabling it would split the architecture into disconnected segments.",
        actions: [
          "Add a redundant path or fallback service",
          "Ensure high availability (multi-AZ, reserved concurrency)",
          "Create a disaster recovery runbook for this specific node",
          "Add health check monitoring with sub-minute resolution",
          "Consider deploying this in a separate account for blast radius isolation",
        ],
      });
    }

    // --- Runtime age ---
    if (signals.runtime_age >= 1.0) {
      recs.push({
        severity: "critical",
        category: "runtime-lifecycle",
        signal: "runtime_age",
        title: "Deprecated runtime: " + (props.runtime || "unknown"),
        detail:
          "This function uses a runtime that has reached end-of-life. " +
          "AWS may disable creation of new functions and eventually block invocations.",
        actions: [
          "Upgrade to the latest supported runtime version immediately",
          "Test with the new runtime in a staging environment first",
          "Check for breaking changes in the runtime changelog",
          "Update any layers/dependencies that may be runtime-specific",
        ],
      });
    } else if (signals.runtime_age >= 0.5) {
      recs.push({
        severity: "medium",
        category: "runtime-lifecycle",
        signal: "runtime_age",
        title: "Aging runtime: " + (props.runtime || "unknown"),
        detail:
          "This runtime version is approaching end-of-life. Plan an upgrade " +
          "before it becomes unsupported.",
        actions: [
          "Schedule a runtime upgrade within the next quarter",
          "Review AWS runtime support policy for EOL dates",
        ],
      });
    }

    // --- Timeout risk ---
    if (signals.timeout_risk >= 0.7) {
      if (props.timeout === undefined || props.timeout === null) {
        recs.push({
          severity: "high",
          category: "configuration",
          signal: "timeout_risk",
          title: "No timeout configured",
          detail:
            "This function has no explicit timeout. It may run indefinitely, " +
            "consuming resources and blocking downstream processes.",
          actions: [
            "Set an appropriate timeout based on expected execution duration",
            "Add CloudWatch alarms for duration metrics",
          ],
        });
      } else {
        recs.push({
          severity: "high",
          category: "configuration",
          signal: "timeout_risk",
          title: "Very long timeout: " + props.timeout + "s",
          detail:
            "A timeout of " +
            props.timeout +
            " seconds is very long. This " +
            "can cause cascading timeouts in upstream services and may indicate " +
            "the function is doing too much work.",
          actions: [
            "Consider breaking this into smaller functions chained via Step Functions",
            "Optimize the workload to reduce execution time",
            "If long execution is required, use ECS/Fargate instead of Lambda",
            "Ensure upstream callers have independent timeout settings",
          ],
        });
      }
    } else if (signals.timeout_risk >= 0.3) {
      recs.push({
        severity: "low",
        category: "configuration",
        signal: "timeout_risk",
        title: "Moderate timeout: " + (props.timeout || "?") + "s",
        detail:
          "The timeout is in a moderate range. Monitor execution duration " +
          "to ensure it stays well below the limit.",
        actions: [
          "Add a CloudWatch alarm for p99 duration approaching the timeout",
        ],
      });
    }

    // --- Isolated node ---
    if (signals.isolated > 0) {
      recs.push({
        severity: "medium",
        category: "connectivity",
        signal: "isolated",
        title: "Isolated node with no connections",
        detail:
          "This node has no incoming or outgoing connections in the discovered " +
          "graph. It may be orphaned, unused, or triggered through a mechanism " +
          "not captured by Hawk.",
        actions: [
          "Verify this resource is still needed",
          "Check if it is invoked via SDK calls, CloudFormation custom resources, or other undiscovered triggers",
          "If unused, consider removing it to reduce surface area and cost",
          "Tag it with an owner for accountability",
        ],
      });
    }

    // --- Single connection ---
    if (signals.single_conn >= 1.0 && signals.isolated === 0) {
      recs.push({
        severity: "low",
        category: "resilience",
        signal: "single_conn",
        title: "Single connection point",
        detail:
          "This node has only one connection. If that link fails, this node " +
          "becomes effectively orphaned.",
        actions: [
          "Evaluate if an additional trigger path or consumer would improve resilience",
          "Ensure the single connection has retry and DLQ handling",
        ],
      });
    }

    // --- Missing DLQ ---
    if (signals.missing_dlq >= 0.6) {
      recs.push({
        severity: "high",
        category: "error-handling",
        signal: "missing_dlq",
        title: "No dead-letter queue detected",
        detail:
          "Failed messages or invocations may be silently lost. " +
          "A dead-letter queue captures failures for retry or investigation.",
        actions: [
          "Configure a DLQ on the SQS queue or Lambda event source mapping",
          "Set up CloudWatch alarms on the DLQ message count",
          "Implement a DLQ processor to alert on or retry failed messages",
        ],
      });
    } else if (signals.missing_dlq >= 0.3) {
      recs.push({
        severity: "medium",
        category: "error-handling",
        signal: "missing_dlq",
        title: "Error handling could be improved",
        detail:
          "Consider adding explicit error handling, destination configuration, " +
          "or a dead-letter queue to capture failures.",
        actions: [
          "Add Lambda Destinations for async invocations",
          "Configure on-failure destinations to an SQS queue or SNS topic",
        ],
      });
    }

    // Sort by severity
    const severityOrder = { critical: 0, high: 1, medium: 2, low: 3 };
    recs.sort(
      (a, b) =>
        (severityOrder[a.severity] || 9) - (severityOrder[b.severity] || 9),
    );

    return recs;
  }

  /**
   * Generate recommendations for ALL nodes. Returns Map<nodeId, recs[]>.
   */
  function allRecommendations() {
    const results = new Map();
    for (const id of nodeIds) {
      const recs = recommendations(id);
      if (recs.length > 0) {
        results.set(id, recs);
      }
    }
    return results;
  }

  // =========================================================================
  // PUBLIC API
  // =========================================================================

  return {
    // Core data access
    getNode: function (id) {
      return nodeMap.get(id);
    },
    getNodeIds: function () {
      return nodeIds.slice();
    },
    getFanIn: function (id) {
      return (adjIn.get(id) || []).length;
    },
    getFanOut: function (id) {
      return (adjOut.get(id) || []).length;
    },

    // 1. Risk scores
    computeNodeRisk: computeNodeRisk,
    computeAllRiskScores: computeAllRiskScores,
    riskTier: riskTier,
    RISK_WEIGHTS: RISK_WEIGHTS,

    // 2. Articulation points
    findArticulationPoints: findArticulationPoints,

    // 3. Longest critical path
    longestCriticalPath: longestCriticalPath,

    // 4. Blast radius
    blastRadius: blastRadius,

    // 5. Cluster health
    clusterHealth: clusterHealth,

    // 6. What-if simulation
    whatIfRemove: whatIfRemove,

    // 7. Graph health score
    graphHealthScore: graphHealthScore,

    // 8. Recommendations
    recommendations: recommendations,
    allRecommendations: allRecommendations,
  };
}

// Export for both browser and Node.js
if (typeof module !== "undefined" && module.exports) {
  module.exports = HawkRiskEngine;
}
