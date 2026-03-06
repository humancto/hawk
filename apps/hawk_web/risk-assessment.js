// ============================================================================
// HAWK Risk Assessment Dashboard — Premium UI Components
// Append CSS via injectRiskAssessmentStyles(), then call component builders.
// All DOM construction uses createElement — zero innerHTML.
// ============================================================================

// ===== CSS INJECTION =====
function injectRiskAssessmentStyles() {
  const style = document.createElement("style");
  style.textContent = `

/* =========================================================================
   RISK ASSESSMENT — CSS
   Premium enterprise dashboard styles for Hawk Infrastructure Observatory
   ========================================================================= */

/* --- CSS Custom Properties for Animated Gauge Counter --- */
@property --gauge-value {
  syntax: '<number>';
  inherits: false;
  initial-value: 0;
}

@property --gauge-angle {
  syntax: '<angle>';
  inherits: false;
  initial-value: 0deg;
}

/* --- Keyframes --- */

@keyframes gaugeCountUp {
  from { --gauge-value: 0; }
}

@keyframes gaugeRingFill {
  from { --gauge-angle: 0deg; }
}

@keyframes riskItemSlideIn {
  from {
    opacity: 0;
    transform: translateX(12px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes criticalPulseGlow {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(245, 91, 91, 0),
                0 0 0 0 rgba(245, 91, 91, 0);
  }
  50% {
    box-shadow: 0 0 8px 2px rgba(245, 91, 91, 0.4),
                0 0 20px 4px rgba(245, 91, 91, 0.15);
  }
}

@keyframes criticalTextPulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes fadeInScale {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes shimmer {
  0% { background-position: -200% center; }
  100% { background-position: 200% center; }
}

@keyframes blastPulse {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(245, 91, 91, 0.4);
  }
  50% {
    box-shadow: 0 0 0 8px rgba(245, 91, 91, 0);
  }
}

@keyframes modalBackdropIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes modalContentIn {
  from {
    opacity: 0;
    transform: translateY(24px) scale(0.97);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes bottomBarSlideIn {
  from {
    opacity: 0;
    transform: translateY(100%);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes numberTick {
  0% { transform: translateY(8px); opacity: 0; }
  100% { transform: translateY(0); opacity: 1; }
}

@keyframes ringRotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* =========================================================================
   1. ASSESSMENT TAB — Right Sidebar (280px)
   ========================================================================= */

/* -- Assessment Tab Button -- */
.tab-btn[data-tab="assessment"] {
  position: relative;
}

.tab-btn[data-tab="assessment"].has-critical::after {
  content: '';
  position: absolute;
  top: 4px;
  right: 4px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent-red);
  animation: criticalPulseGlow 2s ease-in-out infinite;
}

/* -- 1a. Graph Health Gauge Card -- */
.health-gauge-card {
  position: relative;
  padding: 16px 12px;
  background: linear-gradient(135deg, rgba(34, 38, 51, 0.6), rgba(26, 29, 39, 0.9));
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  margin-bottom: 12px;
  overflow: hidden;
  animation: fadeInUp 0.5s ease both;
}

/* Subtle gradient border effect */
.health-gauge-card::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: var(--radius-lg);
  padding: 1px;
  background: linear-gradient(
    135deg,
    rgba(91, 245, 163, 0.3),
    rgba(245, 224, 91, 0.2),
    rgba(245, 91, 91, 0.3)
  );
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  pointer-events: none;
}

.health-gauge-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

/* Circular gauge using conic-gradient */
.health-circular-gauge {
  position: relative;
  width: 120px;
  height: 120px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  --gauge-angle: 0deg;
  background: conic-gradient(
    from 135deg,
    var(--accent-green) 0deg,
    var(--accent-yellow) calc(var(--gauge-angle) * 0.5),
    var(--accent-orange) calc(var(--gauge-angle) * 0.75),
    var(--accent-red) var(--gauge-angle),
    var(--bg-tertiary) var(--gauge-angle),
    var(--bg-tertiary) 270deg
  );
  animation: gaugeRingFill 1.5s cubic-bezier(0.34, 1.56, 0.64, 1) both;
  transition: --gauge-angle 1s ease;
}

.health-circular-gauge::before {
  content: '';
  position: absolute;
  inset: 8px;
  border-radius: 50%;
  background: var(--bg-primary);
}

.health-circular-gauge::after {
  content: '';
  position: absolute;
  inset: 6px;
  border-radius: 50%;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.03);
}

.gauge-center {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  line-height: 1;
}

.gauge-value {
  font-family: var(--font-mono);
  font-size: 1.8rem;
  font-weight: 700;
  color: var(--text-primary);
  --gauge-value: 0;
  counter-reset: gauge var(--gauge-value);
  animation: gaugeCountUp 1.5s ease both;
}

.gauge-value::after {
  content: counter(gauge);
}

.gauge-label {
  font-family: var(--font-mono);
  font-size: 0.55rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 2px;
  margin-top: 4px;
}

.gauge-severity-label {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  font-weight: 600;
  margin-top: 2px;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.gauge-severity-label.severity-good { color: var(--accent-green); }
.gauge-severity-label.severity-fair { color: var(--accent-yellow); }
.gauge-severity-label.severity-poor { color: var(--accent-orange); }
.gauge-severity-label.severity-critical { color: var(--accent-red); }

/* Mini metric cards 2x2 grid */
.health-metrics-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  width: 100%;
}

.health-metric-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 8px 10px;
  text-align: center;
  transition: all 0.2s ease;
  animation: fadeInUp 0.5s ease both;
}

.health-metric-card:hover {
  background: var(--bg-hover);
  border-color: var(--border-active);
  transform: translateY(-1px);
}

.health-metric-value {
  font-family: var(--font-mono);
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
  animation: numberTick 0.4s ease both;
}

.health-metric-value.metric-warning { color: var(--accent-orange); }
.health-metric-value.metric-danger { color: var(--accent-red); }
.health-metric-value.metric-good { color: var(--accent-green); }

.health-metric-label {
  font-family: var(--font-mono);
  font-size: 0.55rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-top: 2px;
  line-height: 1.3;
}

/* -- 1b. Top Risks List -- */
.top-risks-section {
  max-height: 340px;
  overflow-y: auto;
  padding-right: 2px;
}

.risk-list-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 8px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all 0.18s ease;
  animation: riskItemSlideIn 0.35s ease both;
  border: 1px solid transparent;
}

.risk-list-item:hover {
  background: var(--bg-hover);
  border-color: var(--border);
  transform: translateX(2px);
}

.risk-list-item:active {
  transform: translateX(0);
}

.risk-list-score {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  font-family: var(--font-mono);
  font-size: 0.65rem;
  font-weight: 700;
  flex-shrink: 0;
  transition: all 0.2s ease;
}

.risk-list-score.score-critical {
  background: rgba(245, 91, 91, 0.15);
  color: var(--accent-red);
  border: 2px solid rgba(245, 91, 91, 0.5);
  animation: criticalPulseGlow 2.5s ease-in-out infinite;
}

.risk-list-score.score-high {
  background: rgba(245, 160, 91, 0.15);
  color: var(--accent-orange);
  border: 2px solid rgba(245, 160, 91, 0.4);
}

.risk-list-score.score-medium {
  background: rgba(245, 224, 91, 0.15);
  color: var(--accent-yellow);
  border: 2px solid rgba(245, 224, 91, 0.4);
}

.risk-list-score.score-low {
  background: rgba(91, 245, 163, 0.15);
  color: var(--accent-green);
  border: 2px solid rgba(91, 245, 163, 0.4);
}

.risk-list-body {
  flex: 1;
  min-width: 0;
}

.risk-list-header {
  display: flex;
  align-items: center;
  gap: 5px;
}

.risk-list-name {
  font-family: var(--font-mono);
  font-size: 0.72rem;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.risk-list-kind-icon {
  font-size: 0.75rem;
  flex-shrink: 0;
  opacity: 0.7;
}

.risk-list-reason {
  font-size: 0.6rem;
  color: var(--text-muted);
  margin-top: 1px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.risk-list-rank {
  font-family: var(--font-mono);
  font-size: 0.55rem;
  color: var(--text-muted);
  opacity: 0.5;
  flex-shrink: 0;
  width: 16px;
  text-align: right;
}

/* -- 1c. Recommendations -- */
.recommendations-section {
  margin-top: 4px;
}

.rec-severity-group {
  margin-bottom: 6px;
  animation: fadeInUp 0.4s ease both;
}

.rec-severity-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: background 0.15s ease;
  user-select: none;
}

.rec-severity-header:hover {
  background: var(--bg-hover);
}

.rec-severity-header-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.rec-severity-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.rec-severity-dot.dot-critical {
  background: var(--accent-red);
  animation: criticalPulseGlow 2s ease-in-out infinite;
}

.rec-severity-dot.dot-high { background: var(--accent-orange); }
.rec-severity-dot.dot-medium { background: var(--accent-yellow); }

.rec-severity-label {
  font-family: var(--font-mono);
  font-size: 0.65rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-secondary);
}

.rec-severity-count {
  font-family: var(--font-mono);
  font-size: 0.6rem;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  padding: 1px 6px;
  border-radius: 8px;
}

.rec-severity-chevron {
  font-size: 0.6rem;
  color: var(--text-muted);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.rec-severity-header.expanded .rec-severity-chevron {
  transform: rotate(90deg);
}

.rec-items-container {
  max-height: 0;
  overflow: hidden;
  transition: max-height 0.3s ease, opacity 0.2s ease;
  opacity: 0;
}

.rec-items-container.expanded {
  max-height: 1000px;
  opacity: 1;
}

.rec-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 8px 6px 24px;
  border-radius: var(--radius-sm);
  transition: background 0.15s ease;
  animation: riskItemSlideIn 0.3s ease both;
}

.rec-item:hover {
  background: var(--bg-hover);
}

.rec-item-icon {
  font-size: 0.8rem;
  flex-shrink: 0;
  margin-top: 1px;
}

.rec-item-body {
  flex: 1;
  min-width: 0;
}

.rec-item-desc {
  font-size: 0.7rem;
  color: var(--text-secondary);
  line-height: 1.45;
}

.rec-item-node {
  display: inline;
  font-family: var(--font-mono);
  font-size: 0.65rem;
  color: var(--accent-cyan);
  cursor: pointer;
  transition: color 0.15s ease;
  text-decoration: none;
  border-bottom: 1px dotted rgba(78, 205, 196, 0.3);
}

.rec-item-node:hover {
  color: var(--accent-blue);
  border-bottom-color: var(--accent-blue);
}

/* =========================================================================
   2. RISK OVERLAY TOOLBAR CONTROLS
   ========================================================================= */

.toolbar-risk-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.tbtn-risk-view {
  position: relative;
  display: flex;
  align-items: center;
  gap: 5px;
  background: var(--bg-secondary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: 0.7rem;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.tbtn-risk-view:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tbtn-risk-view.active {
  background: linear-gradient(135deg, rgba(245, 91, 91, 0.2), rgba(245, 160, 91, 0.2));
  color: var(--accent-orange);
  border-color: rgba(245, 160, 91, 0.4);
}

.tbtn-risk-view .flame-icon {
  font-size: 0.85rem;
  transition: filter 0.2s ease;
}

.tbtn-risk-view.active .flame-icon {
  filter: drop-shadow(0 0 4px rgba(245, 91, 91, 0.6));
}

.tbtn-blast-radius {
  display: flex;
  align-items: center;
  gap: 5px;
  background: var(--bg-secondary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: 0.7rem;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.tbtn-blast-radius:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tbtn-blast-radius.active {
  background: linear-gradient(135deg, rgba(245, 91, 91, 0.25), rgba(245, 91, 91, 0.1));
  color: var(--accent-red);
  border-color: rgba(245, 91, 91, 0.5);
  animation: blastPulse 2s ease-in-out infinite;
}

.tbtn-blast-radius .blast-icon {
  font-size: 0.85rem;
}

.severity-filter-select {
  background: var(--bg-secondary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  padding: 4px 24px 4px 8px;
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 0.7rem;
  cursor: pointer;
  outline: none;
  transition: all 0.15s ease;
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%235d6280'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
}

.severity-filter-select:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.severity-filter-select:focus {
  border-color: var(--accent-blue);
}

.severity-filter-select option {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

/* =========================================================================
   3. BLAST RADIUS PANEL — Overlay Badge
   ========================================================================= */

.blast-radius-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 60;
  display: none;
  align-items: center;
  gap: 10px;
  background: linear-gradient(135deg, rgba(245, 91, 91, 0.92), rgba(200, 50, 50, 0.92));
  color: #fff;
  padding: 8px 16px;
  border-radius: 24px;
  font-family: var(--font-mono);
  font-size: 0.72rem;
  box-shadow:
    0 4px 24px rgba(245, 91, 91, 0.35),
    0 0 0 1px rgba(255, 255, 255, 0.1) inset;
  backdrop-filter: blur(8px);
  animation: fadeInScale 0.3s ease both;
}

.blast-radius-badge.visible {
  display: flex;
}

.blast-radius-badge-icon {
  font-size: 1rem;
  animation: criticalTextPulse 1.5s ease-in-out infinite;
}

.blast-radius-badge-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.blast-radius-badge-title {
  font-weight: 600;
  font-size: 0.72rem;
  letter-spacing: 0.5px;
}

.blast-radius-badge-detail {
  font-size: 0.6rem;
  opacity: 0.85;
}

.blast-radius-badge-stats {
  display: flex;
  gap: 12px;
  margin-left: 4px;
}

.blast-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.blast-stat-value {
  font-size: 1rem;
  font-weight: 700;
}

.blast-stat-label {
  font-size: 0.5rem;
  opacity: 0.75;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.blast-radius-badge-close {
  background: rgba(255, 255, 255, 0.15);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #fff;
  cursor: pointer;
  padding: 2px 8px;
  border-radius: 12px;
  font-family: var(--font-mono);
  font-size: 0.6rem;
  transition: all 0.15s ease;
}

.blast-radius-badge-close:hover {
  background: rgba(255, 255, 255, 0.25);
}

/* =========================================================================
   4. WHAT-IF MODAL
   ========================================================================= */

.whatif-modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 600;
  background: rgba(0, 0, 0, 0.65);
  display: none;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(4px);
  animation: modalBackdropIn 0.2s ease both;
}

.whatif-modal-overlay.visible {
  display: flex;
}

.whatif-modal {
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  width: 480px;
  max-width: 90vw;
  box-shadow:
    0 24px 80px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(255, 255, 255, 0.03) inset;
  animation: modalContentIn 0.35s cubic-bezier(0.16, 1, 0.3, 1) both;
  overflow: hidden;
}

.whatif-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border);
  background: linear-gradient(135deg, rgba(168, 123, 255, 0.08), transparent);
}

.whatif-modal-header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.whatif-modal-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  background: rgba(168, 123, 255, 0.15);
  border: 1px solid rgba(168, 123, 255, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
}

.whatif-modal-title {
  font-family: var(--font-mono);
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-primary);
}

.whatif-modal-subtitle {
  font-size: 0.65rem;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.whatif-modal-close {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 1.2rem;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  transition: all 0.15s;
}

.whatif-modal-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* Target node info */
.whatif-target {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 18px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.whatif-target-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.3rem;
  flex-shrink: 0;
}

.whatif-target-info {
  flex: 1;
  min-width: 0;
}

.whatif-target-name {
  font-family: var(--font-mono);
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.whatif-target-kind {
  font-family: var(--font-mono);
  font-size: 0.6rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-top: 2px;
}

.whatif-remove-badge {
  background: rgba(245, 91, 91, 0.15);
  color: var(--accent-red);
  border: 1px solid rgba(245, 91, 91, 0.3);
  padding: 3px 10px;
  border-radius: 12px;
  font-family: var(--font-mono);
  font-size: 0.6rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  flex-shrink: 0;
}

/* Impact summary */
.whatif-impact {
  padding: 18px;
}

.whatif-impact-title {
  font-family: var(--font-mono);
  font-size: 0.65rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: var(--text-muted);
  margin-bottom: 12px;
}

.whatif-impact-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 10px;
  margin-bottom: 16px;
}

.whatif-impact-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 12px;
  text-align: center;
  transition: all 0.2s ease;
  animation: fadeInUp 0.4s ease both;
}

.whatif-impact-card.impact-danger {
  border-color: rgba(245, 91, 91, 0.3);
  background: rgba(245, 91, 91, 0.05);
}

.whatif-impact-card.impact-warning {
  border-color: rgba(245, 160, 91, 0.3);
  background: rgba(245, 160, 91, 0.05);
}

.whatif-impact-value {
  font-family: var(--font-mono);
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1;
}

.whatif-impact-value.val-danger { color: var(--accent-red); }
.whatif-impact-value.val-warning { color: var(--accent-orange); }

.whatif-impact-label {
  font-family: var(--font-mono);
  font-size: 0.55rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-top: 6px;
  line-height: 1.3;
}

/* Before/after comparison */
.whatif-comparison {
  margin-top: 4px;
}

.whatif-comparison-title {
  font-family: var(--font-mono);
  font-size: 0.6rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.whatif-comparison-table {
  width: 100%;
  border-collapse: collapse;
}

.whatif-comparison-row {
  display: flex;
  align-items: center;
  padding: 5px 8px;
  border-radius: var(--radius-sm);
  font-size: 0.72rem;
}

.whatif-comparison-row:nth-child(odd) {
  background: var(--bg-secondary);
}

.whatif-comparison-metric {
  flex: 1;
  color: var(--text-secondary);
  font-family: var(--font-sans);
}

.whatif-comparison-before {
  width: 50px;
  text-align: right;
  font-family: var(--font-mono);
  color: var(--text-muted);
  font-size: 0.7rem;
}

.whatif-comparison-arrow {
  width: 30px;
  text-align: center;
  color: var(--text-muted);
  font-size: 0.6rem;
}

.whatif-comparison-after {
  width: 50px;
  text-align: right;
  font-family: var(--font-mono);
  font-weight: 600;
  font-size: 0.7rem;
}

.whatif-comparison-after.val-worse { color: var(--accent-red); }
.whatif-comparison-after.val-better { color: var(--accent-green); }
.whatif-comparison-after.val-same { color: var(--text-muted); }

/* Modal footer */
.whatif-modal-footer {
  display: flex;
  gap: 8px;
  padding: 14px 18px;
  border-top: 1px solid var(--border);
  justify-content: flex-end;
  background: var(--bg-secondary);
}

.whatif-btn {
  padding: 8px 20px;
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  border: 1px solid var(--border);
}

.whatif-btn-cancel {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.whatif-btn-cancel:hover {
  background: var(--bg-hover);
}

.whatif-btn-confirm {
  background: linear-gradient(135deg, var(--accent-red), #d44040);
  color: #fff;
  border-color: transparent;
}

.whatif-btn-confirm:hover {
  filter: brightness(1.1);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(245, 91, 91, 0.3);
}

.whatif-btn-confirm:active {
  transform: translateY(0);
}

/* =========================================================================
   5. EXPORT REPORT MODAL — Full-screen
   ========================================================================= */

.export-report-overlay {
  position: fixed;
  inset: 0;
  z-index: 700;
  background: rgba(0, 0, 0, 0.75);
  display: none;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(6px);
  animation: modalBackdropIn 0.25s ease both;
}

.export-report-overlay.visible {
  display: flex;
}

.export-report-modal {
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  width: 90vw;
  max-width: 900px;
  height: 85vh;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow:
    0 32px 100px rgba(0, 0, 0, 0.6),
    0 0 0 1px rgba(255, 255, 255, 0.03) inset;
  animation: modalContentIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
  overflow: hidden;
}

.export-report-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  background: linear-gradient(135deg, rgba(91, 138, 245, 0.06), transparent);
}

.export-report-header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.export-report-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  background: rgba(91, 138, 245, 0.15);
  border: 1px solid rgba(91, 138, 245, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
}

.export-report-title {
  font-family: var(--font-mono);
  font-size: 0.9rem;
  font-weight: 600;
}

.export-report-close {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 1.2rem;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  transition: all 0.15s;
}

.export-report-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.export-report-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.export-report-preview {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 20px 24px;
  font-family: var(--font-mono);
  font-size: 0.72rem;
  color: var(--text-secondary);
  line-height: 1.7;
  white-space: pre-wrap;
  word-break: break-word;
  min-height: 300px;
}

.export-report-preview .report-h1 {
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.export-report-preview .report-h2 {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--accent-cyan);
  margin-top: 16px;
  margin-bottom: 4px;
}

.export-report-preview .report-h3 {
  font-size: 0.78rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-top: 10px;
  margin-bottom: 2px;
}

.export-report-preview .report-divider {
  border: none;
  border-top: 1px solid var(--border);
  margin: 12px 0;
}

.export-report-footer {
  display: flex;
  gap: 8px;
  padding: 14px 20px;
  border-top: 1px solid var(--border);
  justify-content: flex-end;
  flex-shrink: 0;
  background: var(--bg-secondary);
}

.export-report-btn {
  padding: 8px 20px;
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: 6px;
}

.export-report-btn-copy {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.export-report-btn-copy:hover {
  background: var(--bg-hover);
  border-color: var(--border-active);
}

.export-report-btn-copy.copied {
  background: rgba(91, 245, 163, 0.1);
  color: var(--accent-green);
  border-color: rgba(91, 245, 163, 0.3);
}

.export-report-btn-download {
  background: linear-gradient(135deg, var(--accent-blue), #4a7ae5);
  color: #fff;
  border-color: transparent;
}

.export-report-btn-download:hover {
  filter: brightness(1.1);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(91, 138, 245, 0.3);
}

/* =========================================================================
   6. ASSESSMENT BOTTOM BAR
   ========================================================================= */

.bottombar-risk-section {
  display: none;
  align-items: center;
  gap: 12px;
  animation: bottomBarSlideIn 0.3s ease both;
}

.bottombar-risk-section.visible {
  display: flex;
}

.bottombar-risk-sep {
  width: 1px;
  height: 18px;
  background: var(--border);
}

.bottombar-health {
  display: flex;
  align-items: center;
  gap: 5px;
  font-family: var(--font-mono);
  font-size: 0.65rem;
}

.bottombar-health-label {
  color: var(--text-muted);
}

.bottombar-health-value {
  font-weight: 700;
  transition: color 0.3s ease;
}

.bottombar-health-value.health-good { color: var(--accent-green); }
.bottombar-health-value.health-fair { color: var(--accent-yellow); }
.bottombar-health-value.health-poor { color: var(--accent-orange); }
.bottombar-health-value.health-critical { color: var(--accent-red); }

.bottombar-severity-pill {
  display: flex;
  align-items: center;
  gap: 4px;
  font-family: var(--font-mono);
  font-size: 0.62rem;
  padding: 1px 8px;
  border-radius: 10px;
  transition: all 0.2s ease;
}

.bottombar-severity-pill.pill-critical {
  background: rgba(245, 91, 91, 0.12);
  color: var(--accent-red);
  border: 1px solid rgba(245, 91, 91, 0.25);
}

.bottombar-severity-pill.pill-critical.has-items {
  animation: criticalTextPulse 2s ease-in-out infinite;
}

.bottombar-severity-pill.pill-high {
  background: rgba(245, 160, 91, 0.12);
  color: var(--accent-orange);
  border: 1px solid rgba(245, 160, 91, 0.25);
}

.bottombar-severity-pill.pill-medium {
  background: rgba(245, 224, 91, 0.1);
  color: var(--accent-yellow);
  border: 1px solid rgba(245, 224, 91, 0.2);
}

.bottombar-severity-pill .pill-count {
  font-weight: 700;
}

/* =========================================================================
   UTILITY / SHARED
   ========================================================================= */

/* Risk view heatmap node overlay shimmer */
.risk-heatmap-active .tbtn-risk-view {
  background: linear-gradient(
    90deg,
    rgba(245, 91, 91, 0.1),
    rgba(245, 160, 91, 0.2),
    rgba(245, 91, 91, 0.1)
  );
  background-size: 200% 100%;
  animation: shimmer 3s ease-in-out infinite;
}

/* Smooth transitions for all interactive sidebar elements */
.sidebar-section .risk-list-item,
.sidebar-section .rec-item,
.sidebar-section .rec-severity-header {
  will-change: transform, opacity;
}

/* Assessment-specific scrollbar refinement */
.top-risks-section::-webkit-scrollbar,
.recommendations-section::-webkit-scrollbar {
  width: 4px;
}

.top-risks-section::-webkit-scrollbar-thumb,
.recommendations-section::-webkit-scrollbar-thumb {
  background: rgba(46, 51, 72, 0.6);
  border-radius: 2px;
}

/* Empty state */
.assessment-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 16px;
  text-align: center;
}

.assessment-empty-icon {
  font-size: 2rem;
  opacity: 0.3;
  margin-bottom: 12px;
}

.assessment-empty-text {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--text-muted);
  line-height: 1.5;
}

`;
  document.head.appendChild(style);
}

// ============================================================================
// COMPONENT BUILDERS — All use createElement, zero innerHTML
// ============================================================================

// Reuse the existing createEl helper from the main app
// function createEl(tag, attrs, text) is assumed to exist globally

/**
 * Helper: returns severity class name from a numeric score 0-100
 */
function riskSeverityClass(score) {
  if (score >= 80) return "critical";
  if (score >= 60) return "high";
  if (score >= 35) return "medium";
  return "low";
}

/**
 * Helper: returns the appropriate severity label
 */
function riskSeverityLabel(score) {
  if (score >= 80) return "Critical";
  if (score >= 60) return "High";
  if (score >= 35) return "Medium";
  return "Low";
}

/**
 * Helper: returns gauge severity CSS class
 */
function gaugeSeverityClass(score) {
  if (score >= 75) return "severity-good";
  if (score >= 50) return "severity-fair";
  if (score >= 25) return "severity-poor";
  return "severity-critical";
}

/**
 * Helper: returns gauge severity label text
 */
function gaugeSeverityText(score) {
  if (score >= 75) return "Healthy";
  if (score >= 50) return "Fair";
  if (score >= 25) return "Degraded";
  return "Critical";
}

// =========================================================================
// 1. ASSESSMENT TAB — builds the entire right-sidebar assessment panel
// =========================================================================

/**
 * Builds the Assessment tab content for the right sidebar.
 *
 * @param {Object} options
 * @param {number} options.healthScore - 0-100 overall health
 * @param {number} options.spofsCount - single points of failure
 * @param {number} options.criticalPaths - number of critical paths
 * @param {number} options.avgRisk - average risk score 0-100
 * @param {number} options.isolatedNodes - count of isolated nodes
 * @param {Array} options.topRisks - array of { id, name, kind, score, reason }
 * @param {Object} options.recommendations - { critical: [...], high: [...], medium: [...] }
 * @param {Function} options.onNodeClick - callback(nodeId) when a node is clicked
 * @returns {HTMLElement}
 */
function buildAssessmentTab(options) {
  const {
    healthScore = 0,
    spofsCount = 0,
    criticalPaths = 0,
    avgRisk = 0,
    isolatedNodes = 0,
    topRisks = [],
    recommendations = { critical: [], high: [], medium: [] },
    onNodeClick = function () {},
  } = options;

  const container = document.createElement("div");
  container.className = "tab-content";
  container.id = "tab-assessment";

  // --- 1a. Graph Health Gauge ---
  container.appendChild(
    buildHealthGaugeSection({
      healthScore: healthScore,
      spofsCount: spofsCount,
      criticalPaths: criticalPaths,
      avgRisk: avgRisk,
      isolatedNodes: isolatedNodes,
    }),
  );

  // --- 1b. Top Risks ---
  container.appendChild(
    buildTopRisksSection({
      risks: topRisks,
      onNodeClick: onNodeClick,
    }),
  );

  // --- 1c. Recommendations ---
  container.appendChild(
    buildRecommendationsSection({
      recommendations: recommendations,
      onNodeClick: onNodeClick,
    }),
  );

  return container;
}

/**
 * 1a. Graph Health — Large circular gauge + 2x2 metric cards
 */
function buildHealthGaugeSection(options) {
  const { healthScore, spofsCount, criticalPaths, avgRisk, isolatedNodes } =
    options;

  const section = document.createElement("div");
  section.className = "sidebar-section";

  const title = document.createElement("div");
  title.className = "sidebar-title";
  title.textContent = "Graph Health";
  section.appendChild(title);

  // Card wrapper
  const card = document.createElement("div");
  card.className = "health-gauge-card";

  const wrapper = document.createElement("div");
  wrapper.className = "health-gauge-wrapper";

  // Circular gauge
  const gauge = document.createElement("div");
  gauge.className = "health-circular-gauge";
  // The gauge angle maps score 0-100 to 0-270deg
  const angle = (healthScore / 100) * 270;
  gauge.style.setProperty("--gauge-angle", angle + "deg");

  const center = document.createElement("div");
  center.className = "gauge-center";

  const valueEl = document.createElement("div");
  valueEl.className = "gauge-value";
  // Set CSS custom property for counter animation
  valueEl.style.setProperty("--gauge-value", healthScore);
  // Note: display is handled by ::after counter(gauge) — do NOT set textContent

  const label = document.createElement("div");
  label.className = "gauge-label";
  label.textContent = "Health";

  center.appendChild(valueEl);
  center.appendChild(label);
  gauge.appendChild(center);

  // Severity label below gauge
  const sevLabel = document.createElement("div");
  sevLabel.className =
    "gauge-severity-label " + gaugeSeverityClass(healthScore);
  sevLabel.textContent = gaugeSeverityText(healthScore);

  wrapper.appendChild(gauge);
  wrapper.appendChild(sevLabel);

  // 2x2 metric grid
  const grid = document.createElement("div");
  grid.className = "health-metrics-grid";

  var metrics = [
    {
      value: spofsCount,
      label: "SPOFs",
      className: spofsCount > 0 ? "metric-danger" : "metric-good",
      delay: "0.1s",
    },
    {
      value: criticalPaths,
      label: "Critical Paths",
      className: criticalPaths > 3 ? "metric-warning" : "",
      delay: "0.15s",
    },
    {
      value: avgRisk,
      label: "Avg Risk",
      className:
        avgRisk > 60
          ? "metric-danger"
          : avgRisk > 35
            ? "metric-warning"
            : "metric-good",
      delay: "0.2s",
    },
    {
      value: isolatedNodes,
      label: "Isolated",
      className: isolatedNodes > 5 ? "metric-warning" : "",
      delay: "0.25s",
    },
  ];

  for (var i = 0; i < metrics.length; i++) {
    var m = metrics[i];
    var metricCard = document.createElement("div");
    metricCard.className = "health-metric-card";
    metricCard.style.animationDelay = m.delay;

    var metricValue = document.createElement("div");
    metricValue.className =
      "health-metric-value" + (m.className ? " " + m.className : "");
    metricValue.textContent = String(m.value);

    var metricLabel = document.createElement("div");
    metricLabel.className = "health-metric-label";
    metricLabel.textContent = m.label;

    metricCard.appendChild(metricValue);
    metricCard.appendChild(metricLabel);
    grid.appendChild(metricCard);
  }

  wrapper.appendChild(grid);
  card.appendChild(wrapper);
  section.appendChild(card);

  return section;
}

/**
 * 1b. Top Risks — Scrollable list of riskiest nodes
 */
function buildTopRisksSection(options) {
  var risks = options.risks || [];
  var onNodeClick = options.onNodeClick || function () {};

  var section = document.createElement("div");
  section.className = "sidebar-section";

  var title = document.createElement("div");
  title.className = "sidebar-title";
  title.textContent = "Top Risks (" + risks.length + ")";
  section.appendChild(title);

  if (risks.length === 0) {
    var empty = document.createElement("div");
    empty.className = "assessment-empty";
    var emptyIcon = document.createElement("div");
    emptyIcon.className = "assessment-empty-icon";
    emptyIcon.textContent = "\u2714";
    var emptyText = document.createElement("div");
    emptyText.className = "assessment-empty-text";
    emptyText.textContent = "No significant risks detected";
    empty.appendChild(emptyIcon);
    empty.appendChild(emptyText);
    section.appendChild(empty);
    return section;
  }

  var list = document.createElement("div");
  list.className = "top-risks-section";

  for (var i = 0; i < Math.min(risks.length, 15); i++) {
    var risk = risks[i];
    var sevClass = riskSeverityClass(risk.score);

    var item = document.createElement("div");
    item.className = "risk-list-item";
    item.style.animationDelay = i * 0.04 + "s";

    // Rank number
    var rank = document.createElement("div");
    rank.className = "risk-list-rank";
    rank.textContent = String(i + 1);
    item.appendChild(rank);

    // Score circle
    var scoreCircle = document.createElement("div");
    scoreCircle.className = "risk-list-score score-" + sevClass;
    scoreCircle.textContent = String(risk.score);
    item.appendChild(scoreCircle);

    // Body
    var body = document.createElement("div");
    body.className = "risk-list-body";

    var header = document.createElement("div");
    header.className = "risk-list-header";

    var name = document.createElement("div");
    name.className = "risk-list-name";
    name.textContent = risk.name;
    name.title = risk.name;

    var kindIcon = document.createElement("span");
    kindIcon.className = "risk-list-kind-icon";
    // Use KIND_CONFIG if available (from the main app)
    if (typeof KIND_CONFIG !== "undefined" && KIND_CONFIG[risk.kind]) {
      kindIcon.textContent = KIND_CONFIG[risk.kind].icon;
    } else {
      kindIcon.textContent = "\u25CF";
    }

    header.appendChild(name);
    header.appendChild(kindIcon);
    body.appendChild(header);

    var reason = document.createElement("div");
    reason.className = "risk-list-reason";
    reason.textContent = risk.reason || "";
    reason.title = risk.reason || "";
    body.appendChild(reason);

    item.appendChild(body);

    // Click handler — closure over risk.id
    (function (nodeId) {
      item.addEventListener("click", function () {
        onNodeClick(nodeId);
      });
    })(risk.id);

    list.appendChild(item);
  }

  section.appendChild(list);
  return section;
}

/**
 * 1c. Recommendations — Grouped by severity, collapsible
 */
function buildRecommendationsSection(options) {
  var recs = options.recommendations || { critical: [], high: [], medium: [] };
  var onNodeClick = options.onNodeClick || function () {};

  var section = document.createElement("div");
  section.className = "sidebar-section recommendations-section";

  var title = document.createElement("div");
  title.className = "sidebar-title";
  var totalCount =
    (recs.critical || []).length +
    (recs.high || []).length +
    (recs.medium || []).length;
  title.textContent = "Recommendations (" + totalCount + ")";
  section.appendChild(title);

  if (totalCount === 0) {
    var empty = document.createElement("div");
    empty.className = "assessment-empty";
    var emptyIcon = document.createElement("div");
    emptyIcon.className = "assessment-empty-icon";
    emptyIcon.textContent = "\u2728";
    var emptyText = document.createElement("div");
    emptyText.className = "assessment-empty-text";
    emptyText.textContent = "No recommendations at this time";
    empty.appendChild(emptyIcon);
    empty.appendChild(emptyText);
    section.appendChild(empty);
    return section;
  }

  var groups = [
    {
      key: "critical",
      label: "Critical",
      dotClass: "dot-critical",
      icon: "\u26D4",
      items: recs.critical || [],
      expanded: true,
      delay: "0s",
    },
    {
      key: "high",
      label: "High",
      dotClass: "dot-high",
      icon: "\u26A0",
      items: recs.high || [],
      expanded: false,
      delay: "0.1s",
    },
    {
      key: "medium",
      label: "Medium",
      dotClass: "dot-medium",
      icon: "\u2139",
      items: recs.medium || [],
      expanded: false,
      delay: "0.2s",
    },
  ];

  for (var g = 0; g < groups.length; g++) {
    var group = groups[g];
    if (group.items.length === 0) continue;

    var groupEl = document.createElement("div");
    groupEl.className = "rec-severity-group";
    groupEl.style.animationDelay = group.delay;

    // Header (clickable to expand/collapse)
    var headerEl = document.createElement("div");
    headerEl.className =
      "rec-severity-header" + (group.expanded ? " expanded" : "");

    var headerLeft = document.createElement("div");
    headerLeft.className = "rec-severity-header-left";

    var dot = document.createElement("div");
    dot.className = "rec-severity-dot " + group.dotClass;
    headerLeft.appendChild(dot);

    var labelEl = document.createElement("div");
    labelEl.className = "rec-severity-label";
    labelEl.textContent = group.label;
    headerLeft.appendChild(labelEl);

    var countEl = document.createElement("span");
    countEl.className = "rec-severity-count";
    countEl.textContent = String(group.items.length);
    headerLeft.appendChild(countEl);

    headerEl.appendChild(headerLeft);

    var chevron = document.createElement("span");
    chevron.className = "rec-severity-chevron";
    chevron.textContent = "\u25B6";
    headerEl.appendChild(chevron);

    // Items container
    var itemsContainer = document.createElement("div");
    itemsContainer.className =
      "rec-items-container" + (group.expanded ? " expanded" : "");

    // Toggle expand/collapse
    (function (hdr, container) {
      hdr.addEventListener("click", function () {
        var isExpanded = container.classList.contains("expanded");
        container.classList.toggle("expanded");
        hdr.classList.toggle("expanded");
      });
    })(headerEl, itemsContainer);

    // Build items
    for (var r = 0; r < group.items.length; r++) {
      var rec = group.items[r];

      var recItem = document.createElement("div");
      recItem.className = "rec-item";
      recItem.style.animationDelay = r * 0.05 + "s";

      var recIcon = document.createElement("span");
      recIcon.className = "rec-item-icon";
      recIcon.textContent = group.icon;
      recItem.appendChild(recIcon);

      var recBody = document.createElement("div");
      recBody.className = "rec-item-body";

      var recDesc = document.createElement("div");
      recDesc.className = "rec-item-desc";
      recDesc.textContent = rec.description;

      recBody.appendChild(recDesc);

      // Affected node link
      if (rec.nodeName && rec.nodeId) {
        var nodeLink = document.createElement("span");
        nodeLink.className = "rec-item-node";
        nodeLink.textContent = rec.nodeName;
        nodeLink.title = "Navigate to " + rec.nodeName;
        (function (nid) {
          nodeLink.addEventListener("click", function (e) {
            e.stopPropagation();
            onNodeClick(nid);
          });
        })(rec.nodeId);
        recBody.appendChild(document.createTextNode(" \u2014 "));
        recBody.appendChild(nodeLink);
      }

      recItem.appendChild(recBody);
      itemsContainer.appendChild(recItem);
    }

    groupEl.appendChild(headerEl);
    groupEl.appendChild(itemsContainer);
    section.appendChild(groupEl);
  }

  return section;
}

// =========================================================================
// 2. RISK OVERLAY TOOLBAR CONTROLS
// =========================================================================

/**
 * Builds the risk overlay toolbar control group.
 *
 * @param {Object} options
 * @param {Function} options.onRiskViewToggle - callback(isActive)
 * @param {Function} options.onSeverityFilter - callback(severityLevel)
 * @param {Function} options.onBlastRadiusToggle - callback(isActive)
 * @returns {HTMLElement} - a toolbar-group div to insert into the toolbar
 */
function buildRiskToolbarControls(options) {
  var onRiskViewToggle = options.onRiskViewToggle || function () {};
  var onSeverityFilter = options.onSeverityFilter || function () {};
  var onBlastRadiusToggle = options.onBlastRadiusToggle || function () {};

  var group = document.createElement("div");
  group.className = "toolbar-risk-group";

  // Separator
  var sep = document.createElement("div");
  sep.className = "toolbar-sep";
  group.appendChild(sep);

  // Label
  var label = document.createElement("span");
  label.className = "toolbar-label";
  label.textContent = "Risk";
  group.appendChild(label);

  // Risk View toggle button with flame icon
  var riskBtn = document.createElement("button");
  riskBtn.className = "tbtn-risk-view";
  riskBtn.title = "Toggle risk heatmap overlay (R)";

  var flameIcon = document.createElement("span");
  flameIcon.className = "flame-icon";
  flameIcon.textContent = "\uD83D\uDD25"; // fire emoji
  riskBtn.appendChild(flameIcon);

  var riskLabel = document.createTextNode("Risk View");
  riskBtn.appendChild(riskLabel);

  riskBtn.addEventListener("click", function () {
    var isActive = riskBtn.classList.toggle("active");
    onRiskViewToggle(isActive);
  });
  group.appendChild(riskBtn);

  // Severity filter dropdown
  var select = document.createElement("select");
  select.className = "severity-filter-select";
  select.title = "Filter by severity level";

  var severities = [
    { value: "all", label: "All" },
    { value: "critical", label: "Critical" },
    { value: "high", label: "High" },
    { value: "medium", label: "Medium" },
    { value: "low", label: "Low" },
  ];

  for (var i = 0; i < severities.length; i++) {
    var opt = document.createElement("option");
    opt.value = severities[i].value;
    opt.textContent = severities[i].label;
    select.appendChild(opt);
  }

  select.addEventListener("change", function () {
    onSeverityFilter(select.value);
  });
  group.appendChild(select);

  // Blast Radius toggle
  var blastBtn = document.createElement("button");
  blastBtn.className = "tbtn-blast-radius";
  blastBtn.title = "Toggle blast radius analysis mode (B)";

  var blastIcon = document.createElement("span");
  blastIcon.className = "blast-icon";
  blastIcon.textContent = "\uD83D\uDCA5"; // collision emoji
  blastBtn.appendChild(blastIcon);

  var blastLabel = document.createTextNode("Blast Radius");
  blastBtn.appendChild(blastLabel);

  blastBtn.addEventListener("click", function () {
    var isActive = blastBtn.classList.toggle("active");
    onBlastRadiusToggle(isActive);
  });
  group.appendChild(blastBtn);

  // Store references for external access
  group._riskViewBtn = riskBtn;
  group._severitySelect = select;
  group._blastRadiusBtn = blastBtn;

  return group;
}

// =========================================================================
// 3. BLAST RADIUS BADGE — Overlay on graph
// =========================================================================

/**
 * Builds the blast radius overlay badge.
 *
 * @param {Object} options
 * @param {number} options.affectedCount - number of affected downstream nodes
 * @param {number} options.depth - blast depth
 * @param {string} options.sourceNodeName - name of the source node
 * @param {Function} options.onClose - callback when close is clicked
 * @returns {HTMLElement}
 */
function buildBlastRadiusBadge(options) {
  var affectedCount = options.affectedCount || 0;
  var depth = options.depth || 0;
  var sourceNodeName = options.sourceNodeName || "Unknown";
  var onClose = options.onClose || function () {};

  var badge = document.createElement("div");
  badge.className = "blast-radius-badge";

  // Icon
  var icon = document.createElement("span");
  icon.className = "blast-radius-badge-icon";
  icon.textContent = "\uD83D\uDCA5";
  badge.appendChild(icon);

  // Text block
  var textBlock = document.createElement("div");
  textBlock.className = "blast-radius-badge-text";

  var titleEl = document.createElement("div");
  titleEl.className = "blast-radius-badge-title";
  titleEl.textContent = "Blast Zone: " + sourceNodeName;
  textBlock.appendChild(titleEl);

  var detailEl = document.createElement("div");
  detailEl.className = "blast-radius-badge-detail";
  detailEl.textContent = affectedCount + " nodes affected, depth " + depth;
  textBlock.appendChild(detailEl);

  badge.appendChild(textBlock);

  // Stats
  var stats = document.createElement("div");
  stats.className = "blast-radius-badge-stats";

  var countStat = document.createElement("div");
  countStat.className = "blast-stat";
  var countVal = document.createElement("div");
  countVal.className = "blast-stat-value";
  countVal.textContent = String(affectedCount);
  var countLabel = document.createElement("div");
  countLabel.className = "blast-stat-label";
  countLabel.textContent = "Nodes";
  countStat.appendChild(countVal);
  countStat.appendChild(countLabel);
  stats.appendChild(countStat);

  var depthStat = document.createElement("div");
  depthStat.className = "blast-stat";
  var depthVal = document.createElement("div");
  depthVal.className = "blast-stat-value";
  depthVal.textContent = String(depth);
  var depthLabel = document.createElement("div");
  depthLabel.className = "blast-stat-label";
  depthLabel.textContent = "Depth";
  depthStat.appendChild(depthVal);
  depthStat.appendChild(depthLabel);
  stats.appendChild(depthStat);

  badge.appendChild(stats);

  // Close button
  var closeBtn = document.createElement("button");
  closeBtn.className = "blast-radius-badge-close";
  closeBtn.textContent = "\u2715 Exit";
  closeBtn.addEventListener("click", function () {
    badge.classList.remove("visible");
    onClose();
  });
  badge.appendChild(closeBtn);

  return badge;
}

// =========================================================================
// 4. WHAT-IF MODAL
// =========================================================================

/**
 * Builds the What-If simulation modal.
 *
 * @param {Object} options
 * @param {string} options.nodeName - name of the node being removed
 * @param {string} options.nodeKind - kind of the node
 * @param {string} options.nodeId - ID of the node
 * @param {Object} options.impact
 * @param {number} options.impact.disconnectedSubgraphs
 * @param {number} options.impact.orphanedNodes
 * @param {number} options.impact.affectedDownstream
 * @param {Object} options.before - { nodes, edges, healthScore, avgRisk }
 * @param {Object} options.after - { nodes, edges, healthScore, avgRisk }
 * @param {Function} options.onConfirm - callback when confirm is clicked
 * @param {Function} options.onCancel - callback when cancel is clicked
 * @returns {HTMLElement}
 */
function buildWhatIfModal(options) {
  var nodeName = options.nodeName || "Unknown";
  var nodeKind = options.nodeKind || "Unknown";
  var nodeId = options.nodeId || "";
  var impact = options.impact || {
    disconnectedSubgraphs: 0,
    orphanedNodes: 0,
    affectedDownstream: 0,
  };
  var before = options.before || {
    nodes: 0,
    edges: 0,
    healthScore: 0,
    avgRisk: 0,
  };
  var after = options.after || {
    nodes: 0,
    edges: 0,
    healthScore: 0,
    avgRisk: 0,
  };
  var onConfirm = options.onConfirm || function () {};
  var onCancel = options.onCancel || function () {};

  // Overlay
  var overlay = document.createElement("div");
  overlay.className = "whatif-modal-overlay";

  // Modal container
  var modal = document.createElement("div");
  modal.className = "whatif-modal";

  // --- Header ---
  var header = document.createElement("div");
  header.className = "whatif-modal-header";

  var headerLeft = document.createElement("div");
  headerLeft.className = "whatif-modal-header-left";

  var iconBox = document.createElement("div");
  iconBox.className = "whatif-modal-icon";
  iconBox.textContent = "\uD83E\uDDEA"; // test tube emoji
  headerLeft.appendChild(iconBox);

  var titleBlock = document.createElement("div");
  var titleEl = document.createElement("div");
  titleEl.className = "whatif-modal-title";
  titleEl.textContent = "What-If Analysis";
  var subtitleEl = document.createElement("div");
  subtitleEl.className = "whatif-modal-subtitle";
  subtitleEl.textContent = "Simulate node removal";
  titleBlock.appendChild(titleEl);
  titleBlock.appendChild(subtitleEl);
  headerLeft.appendChild(titleBlock);

  header.appendChild(headerLeft);

  var closeBtn = document.createElement("button");
  closeBtn.className = "whatif-modal-close";
  closeBtn.textContent = "\u2715";
  closeBtn.addEventListener("click", function () {
    overlay.classList.remove("visible");
    onCancel();
  });
  header.appendChild(closeBtn);

  modal.appendChild(header);

  // --- Target node ---
  var target = document.createElement("div");
  target.className = "whatif-target";

  var targetIcon = document.createElement("div");
  targetIcon.className = "whatif-target-icon";
  if (typeof KIND_CONFIG !== "undefined" && KIND_CONFIG[nodeKind]) {
    targetIcon.style.background = KIND_CONFIG[nodeKind].color;
    targetIcon.style.borderRadius = "var(--radius-md)";
    targetIcon.textContent = KIND_CONFIG[nodeKind].icon;
  } else {
    targetIcon.style.background = "var(--bg-tertiary)";
    targetIcon.textContent = "?";
  }
  target.appendChild(targetIcon);

  var targetInfo = document.createElement("div");
  targetInfo.className = "whatif-target-info";

  var targetName = document.createElement("div");
  targetName.className = "whatif-target-name";
  targetName.textContent = nodeName;
  targetInfo.appendChild(targetName);

  var targetKind = document.createElement("div");
  targetKind.className = "whatif-target-kind";
  targetKind.textContent = nodeKind;
  targetInfo.appendChild(targetKind);

  target.appendChild(targetInfo);

  var removeBadge = document.createElement("div");
  removeBadge.className = "whatif-remove-badge";
  removeBadge.textContent = "Removing";
  target.appendChild(removeBadge);

  modal.appendChild(target);

  // --- Impact Summary ---
  var impactSection = document.createElement("div");
  impactSection.className = "whatif-impact";

  var impactTitle = document.createElement("div");
  impactTitle.className = "whatif-impact-title";
  impactTitle.textContent = "Impact Summary";
  impactSection.appendChild(impactTitle);

  var impactGrid = document.createElement("div");
  impactGrid.className = "whatif-impact-grid";

  var impactCards = [
    {
      value: impact.disconnectedSubgraphs,
      label: "Disconnected\nSubgraphs",
      cardClass: impact.disconnectedSubgraphs > 0 ? "impact-danger" : "",
      valueClass: impact.disconnectedSubgraphs > 0 ? "val-danger" : "",
      delay: "0.1s",
    },
    {
      value: impact.orphanedNodes,
      label: "Orphaned\nNodes",
      cardClass: impact.orphanedNodes > 2 ? "impact-warning" : "",
      valueClass: impact.orphanedNodes > 2 ? "val-warning" : "",
      delay: "0.15s",
    },
    {
      value: impact.affectedDownstream,
      label: "Affected\nDownstream",
      cardClass:
        impact.affectedDownstream > 5
          ? "impact-danger"
          : impact.affectedDownstream > 0
            ? "impact-warning"
            : "",
      valueClass:
        impact.affectedDownstream > 5
          ? "val-danger"
          : impact.affectedDownstream > 0
            ? "val-warning"
            : "",
      delay: "0.2s",
    },
  ];

  for (var i = 0; i < impactCards.length; i++) {
    var ic = impactCards[i];
    var card = document.createElement("div");
    card.className =
      "whatif-impact-card" + (ic.cardClass ? " " + ic.cardClass : "");
    card.style.animationDelay = ic.delay;

    var val = document.createElement("div");
    val.className =
      "whatif-impact-value" + (ic.valueClass ? " " + ic.valueClass : "");
    val.textContent = String(ic.value);

    var lab = document.createElement("div");
    lab.className = "whatif-impact-label";
    lab.textContent = ic.label;

    card.appendChild(val);
    card.appendChild(lab);
    impactGrid.appendChild(card);
  }

  impactSection.appendChild(impactGrid);

  // --- Before/After Comparison ---
  var compSection = document.createElement("div");
  compSection.className = "whatif-comparison";

  var compTitle = document.createElement("div");
  compTitle.className = "whatif-comparison-title";
  compTitle.textContent = "Before / After";
  compSection.appendChild(compTitle);

  var compTable = document.createElement("div");
  compTable.className = "whatif-comparison-table";

  var compRows = [
    {
      metric: "Total Nodes",
      before: before.nodes,
      after: after.nodes,
      lowerIsBetter: false,
    },
    {
      metric: "Total Edges",
      before: before.edges,
      after: after.edges,
      lowerIsBetter: false,
    },
    {
      metric: "Health Score",
      before: before.healthScore,
      after: after.healthScore,
      lowerIsBetter: false,
    },
    {
      metric: "Avg Risk",
      before: before.avgRisk,
      after: after.avgRisk,
      lowerIsBetter: true,
    },
  ];

  for (var r = 0; r < compRows.length; r++) {
    var row = compRows[r];
    var rowEl = document.createElement("div");
    rowEl.className = "whatif-comparison-row";

    var metricEl = document.createElement("div");
    metricEl.className = "whatif-comparison-metric";
    metricEl.textContent = row.metric;
    rowEl.appendChild(metricEl);

    var beforeEl = document.createElement("div");
    beforeEl.className = "whatif-comparison-before";
    beforeEl.textContent = String(row.before);
    rowEl.appendChild(beforeEl);

    var arrowEl = document.createElement("div");
    arrowEl.className = "whatif-comparison-arrow";
    arrowEl.textContent = "\u2192";
    rowEl.appendChild(arrowEl);

    var afterEl = document.createElement("div");
    afterEl.className = "whatif-comparison-after";
    afterEl.textContent = String(row.after);

    // Determine color class
    if (row.before === row.after) {
      afterEl.classList.add("val-same");
    } else if (row.lowerIsBetter) {
      afterEl.classList.add(
        row.after < row.before ? "val-better" : "val-worse",
      );
    } else {
      afterEl.classList.add(
        row.after > row.before ? "val-better" : "val-worse",
      );
    }

    rowEl.appendChild(afterEl);
    compTable.appendChild(rowEl);
  }

  compSection.appendChild(compTable);
  impactSection.appendChild(compSection);
  modal.appendChild(impactSection);

  // --- Footer ---
  var footer = document.createElement("div");
  footer.className = "whatif-modal-footer";

  var cancelBtn = document.createElement("button");
  cancelBtn.className = "whatif-btn whatif-btn-cancel";
  cancelBtn.textContent = "Cancel";
  cancelBtn.addEventListener("click", function () {
    overlay.classList.remove("visible");
    onCancel();
  });
  footer.appendChild(cancelBtn);

  var confirmBtn = document.createElement("button");
  confirmBtn.className = "whatif-btn whatif-btn-confirm";
  confirmBtn.textContent = "Simulate Removal";
  confirmBtn.addEventListener("click", function () {
    onConfirm();
  });
  footer.appendChild(confirmBtn);

  modal.appendChild(footer);

  // Close on overlay click
  overlay.addEventListener("click", function (e) {
    if (e.target === overlay) {
      overlay.classList.remove("visible");
      onCancel();
    }
  });

  overlay.appendChild(modal);

  // Public API
  overlay.show = function () {
    overlay.classList.add("visible");
  };
  overlay.hide = function () {
    overlay.classList.remove("visible");
  };

  return overlay;
}

// =========================================================================
// 5. EXPORT REPORT MODAL — Full-screen markdown preview
// =========================================================================

/**
 * Builds the full-screen export report modal.
 *
 * @param {Object} options
 * @param {string} options.markdownContent - the generated markdown text
 * @param {string} options.filename - default filename for download
 * @param {Function} options.onClose - callback when modal is closed
 * @returns {HTMLElement}
 */
function buildExportReportModal(options) {
  var markdownContent = options.markdownContent || "";
  var filename = options.filename || "hawk-risk-report.md";
  var onClose = options.onClose || function () {};

  // Overlay
  var overlay = document.createElement("div");
  overlay.className = "export-report-overlay";

  // Modal
  var modal = document.createElement("div");
  modal.className = "export-report-modal";

  // --- Header ---
  var header = document.createElement("div");
  header.className = "export-report-header";

  var headerLeft = document.createElement("div");
  headerLeft.className = "export-report-header-left";

  var iconBox = document.createElement("div");
  iconBox.className = "export-report-icon";
  iconBox.textContent = "\uD83D\uDCCB"; // clipboard emoji
  headerLeft.appendChild(iconBox);

  var titleEl = document.createElement("div");
  titleEl.className = "export-report-title";
  titleEl.textContent = "Risk Assessment Report";
  headerLeft.appendChild(titleEl);

  header.appendChild(headerLeft);

  var closeBtn = document.createElement("button");
  closeBtn.className = "export-report-close";
  closeBtn.textContent = "\u2715";
  closeBtn.addEventListener("click", function () {
    overlay.classList.remove("visible");
    onClose();
  });
  header.appendChild(closeBtn);

  modal.appendChild(header);

  // --- Body ---
  var body = document.createElement("div");
  body.className = "export-report-body";

  var preview = document.createElement("div");
  preview.className = "export-report-preview";

  // Parse markdown into styled elements (basic rendering)
  var lines = markdownContent.split("\n");
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i];
    var lineEl;

    if (
      line.indexOf("# ") === 0 &&
      line.indexOf("## ") !== 0 &&
      line.indexOf("### ") !== 0
    ) {
      lineEl = document.createElement("div");
      lineEl.className = "report-h1";
      lineEl.textContent = line.substring(2);
    } else if (line.indexOf("## ") === 0 && line.indexOf("### ") !== 0) {
      lineEl = document.createElement("div");
      lineEl.className = "report-h2";
      lineEl.textContent = line.substring(3);
    } else if (line.indexOf("### ") === 0) {
      lineEl = document.createElement("div");
      lineEl.className = "report-h3";
      lineEl.textContent = line.substring(4);
    } else if (line === "---") {
      lineEl = document.createElement("hr");
      lineEl.className = "report-divider";
    } else if (line.trim() === "") {
      lineEl = document.createElement("br");
    } else {
      lineEl = document.createElement("div");
      lineEl.textContent = line;
    }

    preview.appendChild(lineEl);
  }

  body.appendChild(preview);
  modal.appendChild(body);

  // --- Footer ---
  var footer = document.createElement("div");
  footer.className = "export-report-footer";

  // Copy button
  var copyBtn = document.createElement("button");
  copyBtn.className = "export-report-btn export-report-btn-copy";

  var copyIcon = document.createElement("span");
  copyIcon.textContent = "\uD83D\uDCCB";
  copyBtn.appendChild(copyIcon);
  copyBtn.appendChild(document.createTextNode("Copy"));

  copyBtn.addEventListener("click", function () {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(markdownContent).then(function () {
        copyBtn.classList.add("copied");
        // Replace text temporarily
        while (copyBtn.firstChild) copyBtn.removeChild(copyBtn.firstChild);
        var checkIcon = document.createElement("span");
        checkIcon.textContent = "\u2714";
        copyBtn.appendChild(checkIcon);
        copyBtn.appendChild(document.createTextNode("Copied!"));

        setTimeout(function () {
          copyBtn.classList.remove("copied");
          while (copyBtn.firstChild) copyBtn.removeChild(copyBtn.firstChild);
          var icon2 = document.createElement("span");
          icon2.textContent = "\uD83D\uDCCB";
          copyBtn.appendChild(icon2);
          copyBtn.appendChild(document.createTextNode("Copy"));
        }, 2000);
      });
    }
  });
  footer.appendChild(copyBtn);

  // Download button
  var downloadBtn = document.createElement("button");
  downloadBtn.className = "export-report-btn export-report-btn-download";

  var dlIcon = document.createElement("span");
  dlIcon.textContent = "\u2B07";
  downloadBtn.appendChild(dlIcon);
  downloadBtn.appendChild(document.createTextNode("Download"));

  downloadBtn.addEventListener("click", function () {
    var blob = new Blob([markdownContent], {
      type: "text/markdown;charset=utf-8",
    });
    var url = URL.createObjectURL(blob);
    var a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  });
  footer.appendChild(downloadBtn);

  modal.appendChild(footer);

  // Close on overlay click
  overlay.addEventListener("click", function (e) {
    if (e.target === overlay) {
      overlay.classList.remove("visible");
      onClose();
    }
  });

  // ESC to close
  overlay.addEventListener("keydown", function (e) {
    if (e.key === "Escape") {
      overlay.classList.remove("visible");
      onClose();
    }
  });

  overlay.appendChild(modal);

  // Public API
  overlay.show = function () {
    overlay.classList.add("visible");
  };
  overlay.hide = function () {
    overlay.classList.remove("visible");
  };

  return overlay;
}

// =========================================================================
// 6. ASSESSMENT BOTTOM BAR ADDITIONS
// =========================================================================

/**
 * Builds the risk assessment additions for the bottom bar.
 *
 * @param {Object} options
 * @param {number} options.healthScore - 0-100
 * @param {number} options.criticalCount
 * @param {number} options.highCount
 * @param {number} options.mediumCount
 * @returns {HTMLElement}
 */
function buildAssessmentBottomBar(options) {
  var healthScore = options.healthScore || 0;
  var criticalCount = options.criticalCount || 0;
  var highCount = options.highCount || 0;
  var mediumCount = options.mediumCount || 0;

  var section = document.createElement("div");
  section.className = "bottombar-risk-section";

  // Separator from existing bottom bar content
  var sep = document.createElement("div");
  sep.className = "bottombar-risk-sep";
  section.appendChild(sep);

  // Health score
  var healthEl = document.createElement("div");
  healthEl.className = "bottombar-health";

  var healthLabel = document.createElement("span");
  healthLabel.className = "bottombar-health-label";
  healthLabel.textContent = "Health:";
  healthEl.appendChild(healthLabel);

  var healthValue = document.createElement("span");
  healthValue.className = "bottombar-health-value";
  if (healthScore >= 75) healthValue.classList.add("health-good");
  else if (healthScore >= 50) healthValue.classList.add("health-fair");
  else if (healthScore >= 25) healthValue.classList.add("health-poor");
  else healthValue.classList.add("health-critical");
  healthValue.textContent = healthScore + "/100";
  healthEl.appendChild(healthValue);

  section.appendChild(healthEl);

  // Critical pill
  if (criticalCount > 0) {
    var critPill = document.createElement("div");
    critPill.className = "bottombar-severity-pill pill-critical has-items";

    var critCount = document.createElement("span");
    critCount.className = "pill-count";
    critCount.textContent = String(criticalCount);
    critPill.appendChild(critCount);
    critPill.appendChild(document.createTextNode(" Critical"));

    section.appendChild(critPill);
  }

  // High pill
  if (highCount > 0) {
    var highPill = document.createElement("div");
    highPill.className = "bottombar-severity-pill pill-high";

    var hCount = document.createElement("span");
    hCount.className = "pill-count";
    hCount.textContent = String(highCount);
    highPill.appendChild(hCount);
    highPill.appendChild(document.createTextNode(" High"));

    section.appendChild(highPill);
  }

  // Medium pill
  if (mediumCount > 0) {
    var medPill = document.createElement("div");
    medPill.className = "bottombar-severity-pill pill-medium";

    var mCount = document.createElement("span");
    mCount.className = "pill-count";
    mCount.textContent = String(mediumCount);
    medPill.appendChild(mCount);
    medPill.appendChild(document.createTextNode(" Medium"));

    section.appendChild(medPill);
  }

  // Store reference for toggling visibility
  section._healthValue = healthValue;

  return section;
}

// =========================================================================
// INTEGRATION HELPER — Wire everything into the existing Hawk app
// =========================================================================

/**
 * Master integration function. Call after the graph loads.
 * Calculates risk metrics from the Cytoscape graph and wires up all UI.
 *
 * @param {Object} cyInstance - the Cytoscape instance
 * @param {Object} graphData - the raw hawk.json data
 */
function initRiskAssessment(cyInstance, graphData) {
  if (!cyInstance || !graphData) return;

  // Inject styles
  injectRiskAssessmentStyles();

  // --- Compute risk metrics from graph data ---
  var riskData = computeRiskMetrics(cyInstance, graphData);

  // --- 2. Toolbar controls ---
  var toolbar = document.getElementById("toolbar");
  if (toolbar) {
    var toolbarControls = buildRiskToolbarControls({
      onRiskViewToggle: function (active) {
        if (active) {
          document.body.classList.add("risk-heatmap-active");
          applyRiskHeatmap(cyInstance, riskData.nodeRisks);
          bottomBarSection.classList.add("visible");
        } else {
          document.body.classList.remove("risk-heatmap-active");
          removeRiskHeatmap(cyInstance);
          bottomBarSection.classList.remove("visible");
        }
      },
      onSeverityFilter: function (level) {
        filterBySeverity(cyInstance, riskData.nodeRisks, level);
      },
      onBlastRadiusToggle: function (active) {
        // Blast radius mode handled by node click
      },
    });

    // Insert before the spacer
    var spacer = toolbar.querySelector(".toolbar-spacer");
    if (spacer) {
      toolbar.insertBefore(toolbarControls, spacer);
    } else {
      toolbar.appendChild(toolbarControls);
    }
  }

  // --- 1. Assessment tab in right sidebar ---
  var rightSidebar = document.getElementById("right-sidebar");
  var detailPanel = document.getElementById("detail-panel");
  if (rightSidebar && detailPanel) {
    // We patch the showNodeDetail function to add Assessment tab
    var originalShowNodeDetail =
      typeof showNodeDetail === "function" ? showNodeDetail : null;
    if (originalShowNodeDetail) {
      window.showNodeDetail = function (d) {
        originalShowNodeDetail(d);
        addAssessmentTabToPanel(d, riskData, cyInstance);
      };
    }
  }

  // --- 3. Blast radius badge ---
  var cyContainer = document.getElementById("cy-container");
  if (cyContainer) {
    var blastBadge = buildBlastRadiusBadge({
      affectedCount: 0,
      depth: 0,
      sourceNodeName: "",
      onClose: function () {
        removeRiskHeatmap(cyInstance);
      },
    });
    cyContainer.appendChild(blastBadge);
    window._blastBadge = blastBadge;
  }

  // --- 4. What-if modal ---
  var whatIfModal = buildWhatIfModal({
    nodeName: "",
    nodeKind: "",
    nodeId: "",
    impact: {
      disconnectedSubgraphs: 0,
      orphanedNodes: 0,
      affectedDownstream: 0,
    },
    before: { nodes: 0, edges: 0, healthScore: 0, avgRisk: 0 },
    after: { nodes: 0, edges: 0, healthScore: 0, avgRisk: 0 },
    onConfirm: function () {},
    onCancel: function () {},
  });
  document.body.appendChild(whatIfModal);
  window._whatIfModal = whatIfModal;

  // --- 5. Export report modal ---
  var exportModal = buildExportReportModal({
    markdownContent: generateRiskReport(riskData, graphData),
    filename: "hawk-risk-report.md",
    onClose: function () {},
  });
  document.body.appendChild(exportModal);
  window._exportReportModal = exportModal;

  // --- 6. Bottom bar ---
  var bottombar = document.getElementById("bottombar");
  var bottomBarSection = buildAssessmentBottomBar({
    healthScore: riskData.healthScore,
    criticalCount: riskData.severityCounts.critical,
    highCount: riskData.severityCounts.high,
    mediumCount: riskData.severityCounts.medium,
  });
  if (bottombar) {
    var bottomSpacer = bottombar.querySelector(".bottom-spacer");
    if (bottomSpacer) {
      bottombar.insertBefore(bottomBarSection, bottomSpacer);
    } else {
      bottombar.appendChild(bottomBarSection);
    }
  }
}

/**
 * Adds the Assessment tab to the existing detail panel when a node is selected.
 */
function addAssessmentTabToPanel(nodeData, riskData, cyInstance) {
  var panel = document.getElementById("detail-panel");
  if (!panel) return;

  var tabBar = panel.querySelector(".tab-bar");
  if (!tabBar) return;

  // Check if assessment tab already exists
  if (tabBar.querySelector('[data-tab="assessment"]')) return;

  // Add the assessment tab button
  var assessBtn = document.createElement("button");
  assessBtn.className = "tab-btn";
  assessBtn.setAttribute("data-tab", "assessment");
  assessBtn.textContent = "Risk";

  // Add critical indicator if this node has high risk
  var nodeRisk = null;
  for (var i = 0; i < riskData.topRisks.length; i++) {
    if (riskData.topRisks[i].id === nodeData.id) {
      nodeRisk = riskData.topRisks[i];
      break;
    }
  }
  if (nodeRisk && nodeRisk.score >= 80) {
    assessBtn.classList.add("has-critical");
  }

  tabBar.appendChild(assessBtn);

  // Build assessment content
  var assessContent = buildAssessmentTab({
    healthScore: riskData.healthScore,
    spofsCount: riskData.spofsCount,
    criticalPaths: riskData.criticalPaths,
    avgRisk: riskData.avgRisk,
    isolatedNodes: riskData.isolatedNodes,
    topRisks: riskData.topRisks,
    recommendations: riskData.recommendations,
    onNodeClick: function (nodeId) {
      if (
        typeof navigateToNode === "function" &&
        typeof cy !== "undefined" &&
        cy
      ) {
        var node = cy.getElementById(nodeId);
        if (node && !node.empty()) {
          navigateToNode(node);
        }
      }
    },
  });
  panel.appendChild(assessContent);

  // Wire up tab switching for the new tab
  var allTabBtns = tabBar.querySelectorAll(".tab-btn");
  var allTabContents = panel.querySelectorAll(".tab-content");

  assessBtn.addEventListener("click", function () {
    for (var j = 0; j < allTabBtns.length; j++) {
      allTabBtns[j].classList.remove("active");
    }
    for (var k = 0; k < allTabContents.length; k++) {
      allTabContents[k].classList.remove("active");
    }
    assessBtn.classList.add("active");
    assessContent.classList.add("active");
  });

  // Update existing tab handlers to hide assessment content
  for (var t = 0; t < allTabBtns.length; t++) {
    if (allTabBtns[t] !== assessBtn) {
      (function (btn) {
        var origHandler = btn.onclick;
        btn.addEventListener("click", function () {
          assessBtn.classList.remove("active");
          assessContent.classList.remove("active");
        });
      })(allTabBtns[t]);
    }
  }
}

// =========================================================================
// RISK COMPUTATION ENGINE
// =========================================================================

/**
 * Computes risk metrics from the graph.
 * Returns a structured risk data object.
 */
function computeRiskMetrics(cyInstance, graphData) {
  var nodes = cyInstance.nodes();
  var edges = cyInstance.edges();
  var nodeRisks = {};
  var topRisks = [];
  var spofsCount = 0;
  var isolatedNodes = 0;
  var totalRisk = 0;
  var nodeCount = nodes.length;

  // Compute risk score for each node
  nodes.forEach(function (node) {
    var d = node.data();
    var inDeg = node.indegree();
    var outDeg = node.outdegree();
    var totalDeg = inDeg + outDeg;
    var score = 0;
    var reasons = [];

    // High fan-in = bottleneck risk
    if (inDeg >= 5) {
      score += 25;
      reasons.push("High fan-in (" + inDeg + ")");
    } else if (inDeg >= 3) {
      score += 10;
    }

    // High fan-out = blast radius risk
    if (outDeg >= 5) {
      score += 20;
      reasons.push("High fan-out (" + outDeg + ")");
    } else if (outDeg >= 3) {
      score += 8;
    }

    // SPOF: only one path in/out and it is critical
    if (inDeg === 1 && outDeg >= 3) {
      score += 30;
      reasons.push("Single point of failure");
      spofsCount++;
    }

    // Isolated = no redundancy
    if (totalDeg === 0) {
      score += 5;
      isolatedNodes++;
      reasons.push("Isolated node");
    }

    // Lambda-specific risks
    if (d.kind === "Lambda") {
      if (d.props && d.props.timeout && d.props.timeout >= 300) {
        score += 10;
        reasons.push("Long timeout (" + d.props.timeout + "s)");
      }
      if (d.props && d.props.memory_size && d.props.memory_size >= 1024) {
        score += 5;
        reasons.push("High memory (" + d.props.memory_size + "MB)");
      }
    }

    // Bridge node detection (removing it disconnects graph)
    var neighbors = node.neighborhood("node");
    if (totalDeg >= 2) {
      var neighborEdges = 0;
      neighbors.forEach(function (n) {
        neighborEdges += n.edgesWith(neighbors).length;
      });
      if (neighborEdges === 0 && totalDeg >= 3) {
        score += 20;
        reasons.push("Bridge node");
      }
    }

    score = Math.min(100, score);
    totalRisk += score;

    nodeRisks[d.id] = {
      id: d.id,
      name: d.name,
      kind: d.kind,
      score: score,
      reason: reasons.length > 0 ? reasons[0] : "Low risk",
      reasons: reasons,
    };

    if (score > 10) {
      topRisks.push(nodeRisks[d.id]);
    }
  });

  // Sort by score descending
  topRisks.sort(function (a, b) {
    return b.score - a.score;
  });
  topRisks = topRisks.slice(0, 15);

  var avgRisk = nodeCount > 0 ? Math.round(totalRisk / nodeCount) : 0;

  // Health score is inverse of average risk, adjusted
  var healthScore = Math.max(0, Math.min(100, 100 - Math.round(avgRisk * 1.2)));

  // Count critical paths (simplified: paths with all high-risk nodes)
  var criticalPaths = 0;
  topRisks.forEach(function (r) {
    if (r.score >= 60) criticalPaths++;
  });
  criticalPaths = Math.min(criticalPaths, 10);

  // Build recommendations
  var recommendations = { critical: [], high: [], medium: [] };

  topRisks.forEach(function (r) {
    for (var i = 0; i < r.reasons.length; i++) {
      var reason = r.reasons[i];
      var rec = {
        description: "",
        nodeName: r.name,
        nodeId: r.id,
      };

      if (reason.indexOf("Single point of failure") >= 0) {
        rec.description = "Add redundancy or failover for this critical node";
        recommendations.critical.push(rec);
      } else if (reason.indexOf("Bridge node") >= 0) {
        rec.description =
          "This node bridges disconnected parts of the graph. Consider alternative paths";
        recommendations.critical.push(rec);
      } else if (reason.indexOf("High fan-in") >= 0) {
        rec.description = "Reduce dependencies on this bottleneck node";
        recommendations.high.push(rec);
      } else if (reason.indexOf("High fan-out") >= 0) {
        rec.description =
          "High blast radius if this node fails. Consider circuit breakers";
        recommendations.high.push(rec);
      } else if (reason.indexOf("Long timeout") >= 0) {
        rec.description = "Reduce timeout to prevent resource exhaustion";
        recommendations.medium.push(rec);
      } else if (reason.indexOf("Isolated") >= 0) {
        rec.description = "Verify this node is intentionally disconnected";
        recommendations.medium.push(rec);
      }
    }
  });

  // Severity counts
  var severityCounts = { critical: 0, high: 0, medium: 0, low: 0 };
  for (var id in nodeRisks) {
    var s = nodeRisks[id].score;
    if (s >= 80) severityCounts.critical++;
    else if (s >= 60) severityCounts.high++;
    else if (s >= 35) severityCounts.medium++;
    else severityCounts.low++;
  }

  return {
    healthScore: healthScore,
    spofsCount: spofsCount,
    criticalPaths: criticalPaths,
    avgRisk: avgRisk,
    isolatedNodes: isolatedNodes,
    topRisks: topRisks,
    recommendations: recommendations,
    nodeRisks: nodeRisks,
    severityCounts: severityCounts,
  };
}

// =========================================================================
// RISK HEATMAP — Apply/remove risk coloring on graph
// =========================================================================

function applyRiskHeatmap(cyInstance, nodeRisks) {
  cyInstance.nodes().forEach(function (node) {
    var risk = nodeRisks[node.id()];
    if (!risk) return;

    var score = risk.score;
    var color;
    if (score >= 80) color = "#f55b5b";
    else if (score >= 60) color = "#f5a35b";
    else if (score >= 35) color = "#f5e05b";
    else color = "#5bf5a3";

    node.data("_origColor", node.data("color"));
    node.style("background-color", color);

    if (score >= 80) {
      node.style({
        "shadow-blur": 20,
        "shadow-color": "#f55b5b",
        "shadow-opacity": 0.6,
      });
    }
  });
}

function removeRiskHeatmap(cyInstance) {
  cyInstance.nodes().forEach(function (node) {
    var origColor = node.data("_origColor");
    if (origColor) {
      node.style("background-color", origColor);
    }
    node.style({
      "shadow-blur": null,
      "shadow-color": null,
      "shadow-opacity": null,
    });
  });
}

function filterBySeverity(cyInstance, nodeRisks, level) {
  cyInstance.nodes().forEach(function (node) {
    var risk = nodeRisks[node.id()];
    if (!risk) {
      node.style("display", "element");
      return;
    }

    if (level === "all") {
      node.style("display", "element");
      return;
    }

    var sevClass = riskSeverityClass(risk.score);
    if (sevClass === level) {
      node.style("display", "element");
    } else {
      node.style("display", "none");
    }
  });
}

// =========================================================================
// REPORT GENERATION
// =========================================================================

function generateRiskReport(riskData, graphData) {
  var lines = [];
  var now = new Date().toISOString().split("T")[0];

  lines.push("# Hawk Risk Assessment Report");
  lines.push("");
  lines.push("Generated: " + now);
  if (graphData.profile) lines.push("Profile: " + graphData.profile);
  if (graphData.regions) lines.push("Regions: " + graphData.regions.join(", "));
  lines.push("");
  lines.push("---");
  lines.push("");
  lines.push("## Executive Summary");
  lines.push("");
  lines.push(
    "- Overall Health Score: **" +
      riskData.healthScore +
      "/100** (" +
      gaugeSeverityText(riskData.healthScore) +
      ")",
  );
  lines.push("- Single Points of Failure: **" + riskData.spofsCount + "**");
  lines.push("- Critical Paths: **" + riskData.criticalPaths + "**");
  lines.push("- Average Risk Score: **" + riskData.avgRisk + "**");
  lines.push("- Isolated Nodes: **" + riskData.isolatedNodes + "**");
  lines.push("");
  lines.push("### Severity Distribution");
  lines.push("");
  lines.push("| Severity | Count |");
  lines.push("|----------|-------|");
  lines.push("| Critical | " + riskData.severityCounts.critical + " |");
  lines.push("| High     | " + riskData.severityCounts.high + " |");
  lines.push("| Medium   | " + riskData.severityCounts.medium + " |");
  lines.push("| Low      | " + riskData.severityCounts.low + " |");
  lines.push("");
  lines.push("---");
  lines.push("");
  lines.push("## Top Risks");
  lines.push("");

  for (var i = 0; i < riskData.topRisks.length; i++) {
    var r = riskData.topRisks[i];
    lines.push("### " + (i + 1) + ". " + r.name + " (" + r.kind + ")");
    lines.push("");
    lines.push(
      "- Risk Score: **" + r.score + "** (" + riskSeverityLabel(r.score) + ")",
    );
    lines.push("- Factors: " + r.reasons.join(", "));
    lines.push("");
  }

  lines.push("---");
  lines.push("");
  lines.push("## Recommendations");
  lines.push("");

  var allRecs = [
    { label: "Critical", items: riskData.recommendations.critical },
    { label: "High", items: riskData.recommendations.high },
    { label: "Medium", items: riskData.recommendations.medium },
  ];

  for (var g = 0; g < allRecs.length; g++) {
    var group = allRecs[g];
    if (group.items.length === 0) continue;
    lines.push("### " + group.label + " Priority");
    lines.push("");
    for (var j = 0; j < group.items.length; j++) {
      lines.push(
        "- **" + group.items[j].nodeName + "**: " + group.items[j].description,
      );
    }
    lines.push("");
  }

  lines.push("---");
  lines.push("");
  lines.push("*Report generated by Hawk Infrastructure Observatory*");

  return lines.join("\n");
}
