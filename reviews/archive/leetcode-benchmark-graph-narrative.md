

## Primary Chart: Speedup vs Input Size (Line Chart)

The graph should answer **"Does Sifr's advantage hold as problem size grows, and by how much?"**

### Chart Type: Multi-line speedup chart
- **X-axis**: Input size (log scale if sizes span orders of magnitude)
- **Y-axis**: Speedup ratio (Python mean ÷ Sifr mean)
- **One line per problem** (or aggregated per category)
- **Reference line at Y=1**: Sifr=Python baseline**Why speedup over absolute time:**
- Speedup is dimension-agnostic. A 10x speedup is10x whether you're measuring microseconds or milliseconds.
- Developers care about "is it faster and by how much?" not "what's the absolute time?"
- Speedup reveals scaling intuition: flat line = comparable Big-O, rising line = Sifr's Rust codegen has better asymptotics.

**Why NOT throughput:**
- Throughput is useful for hardware benchmarks but obscures the "is Sifr worth it" question. A problem with 1B ops/second throughput sounds great even if Python gets 100B ops/second.

---

## Secondary Charts### 1. Absolute Runtime Overlay (Log Scale)
- **Purpose**: Show absolute performance, not just relative- **X-axis**: Input size
- **Y-axis**: Mean runtime (log scale)
- **Two lines per problem**: Python and Sifr overlaid
- **Why**: Developers want to know absolutes too ("will this run under 100ms?")

### 2. Stability / CV Comparison- **Purpose**: Is the speedup reliable?
- **Type**: Grouped bar chart or dot plot- **Shows**: CV (% coefficient of variation) for Python vs Sifr per problem
- **Why**: A10x speedup that fluctuates 50% is less trustworthy than 8x with low variance### 3. Category Heatmap (scales with problems)
- **Grid**: Categories (rows) × Problems (columns), cell = speedup
- **Color scale**: Diverging from white (speedup=1) to green (Sifr wins)
- **Why**: At scale, a heatmap shows where Sifr helps most without reading individual charts---

## Big-O / Scaling Intuition

This is the most valuable insight a benchmark can deliver:

1. **Slope of speedup curve reveals Big-O behavior**:
   - Flat slope (constant speedup) → Same asymptotics, Sifr wins by constant factor
   - Rising slope → Sifr's compiled code has better asymptotic complexity
   - Declining slope → Python may have pre-computed optimizations at larger sizes

2. **Implementation**: Annotate the chart with detected slope via a simple linear regression on log-log data, label it as "likely same O(n)" or "Sifr appears O(n log n) vs Python O(n²)"

3. **Multiple input sizes per problem are essential** — you cannot show scaling with a single data point.

---

## Organization for Scale (category → problem → size)

```
Category A └── Problem 1
 ├── Chart A.1.1: Speedup vs Input Size
        ├── Chart A.1.2: Absolute Runtime Overlay
        └── Table: Full metrics
  └── Problem2
        └── ...
  └── Category Aggregate
 ├── Heatmap: Category problems × speedup
        └── Summary stats
Category B
 └── ...
```

**Navigation pattern**:
- Left sidebar: collapsible category tree
- Main area: selected problem's charts + table
- Tab bar to switch between chart variants

---

## Misleading Charts to Avoid

| Chart Type | Why It's Bad |
|-----------|--------------|
| Pie chart of time spent | Impossible to read meaningful comparisons; speedup isn't additive |
| Stacked area chart | Obscures individual problem performance; confuses "this much Python time" with "this much slower" |
| Single Y-axis with wildly different magnitudes | If Python=1000ms and Sifr=1ms, you lose Sifr's line in noise |
| 3D charts | Distorts perception; never worth it |
| Speedup as percentage improvement | "400% faster" sounds impressive but "4x speedup" is clearer and standard |

---

## Concrete Static HTML Implementation

```html
<!DOCTYPE html>
<html>
<head>
<style>
  :root {
    --bg: #1a1a2e;
    --surface: #16213e;
    --text: #e0e0e0;
    --muted: #8892a0;
    --sifr: #00d9ff;
    --python: #ffd700;
    --speedup-win: #00ff88;
    --speedup-lose: #ff6b6b;
  }
  
  * { box-sizing: border-box; margin: 0; padding: 0; }
  
  body {
    font-family: system-ui, -apple-system, sans-serif;
    background: var(--bg);
    color: var(--text);
    min-height: 100vh;
    display: grid;
    grid-template-columns: 220px 1fr;
 }
  
  /* Sidebar */
  nav {
    background: var(--surface);
    padding: 1rem;
    border-right: 1px solid #2a2a4a;
    overflow-y: auto;
  }
  
  nav h3 { font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.1em; color: var(--muted); margin: 1rem 0 0.5rem; }
  nav h3:first-child { margin-top: 0; }
  
  nav details summary { cursor: pointer; font-weight: 600; padding: 0.25rem 0; }
  nav ul { list-style: none; margin-left: 1rem; }
  nav a { color: var(--text); text-decoration: none; font-size: 0.875rem; padding: 0.25rem 0; display: block; }
  nav a:hover { color: var(--sifr); }
  nav a.active { color: var(--sifr); border-left: 2px solid var(--sifr); padding-left: 0.5rem; }
  
  /* Main content */
  main { padding: 2rem; overflow-y: auto; }
  
  header { margin-bottom: 2rem; }
  header h1 { font-size: 1.5rem; margin-bottom: 0.5rem; }
  header p { color: var(--muted); font-size: 0.875rem; }
  
  /* Section */
  .section { background: var(--surface); border-radius: 8px; padding: 1.5rem; margin-bottom: 1.5rem; }
  .section h2 { font-size: 1rem; margin-bottom: 1rem; color: var(--sifr); }
  
  /* Chart container */
  .chart-wrap { position: relative; }
  .chart-wrap svg { width: 100%; height: 300px; }
  
  /* Legend */
  .legend { display: flex; gap: 1.5rem; margin-top: 1rem; font-size: 0.875rem; }
  .legend-item { display: flex; align-items: center; gap: 0.5rem; }
  .legend-dot { width: 12px; height: 12px; border-radius: 50%; }
  
  /* Reference line */
  .ref-line { stroke: var(--muted); stroke-dasharray: 4 4; }
  /* Table */
  table { width: 100%; border-collapse: collapse; font-size: 0.875rem; }
  th, td { text-align: left; padding: 0.75rem; border-bottom: 1px solid #2a2a4a; }
  th { font-weight: 600; color: var(--muted); font-size: 0.75rem; text-transform: uppercase; }
  td.numeric { text-align: right; font-variant-numeric: tabular-nums; }
  .speedup { font-weight: 600; }
  .speedup.wins { color: var(--speedup-win); }
  .speedup.loses { color: var(--speedup-lose); }
  
  /* Responsive */
  @media (max-width: 768px) {
    body { grid-template-columns: 1fr; }
    nav { display: none; }
  }
</style>
</head>
<body>
<nav>
  <h3>Two Sum</h3>
  <ul>
    <li><a href="#" class="active">Array -100</a></li>
    <li><a href="#">Array - 1000</a></li>
    <li><a href="#">Array - 10000</a></li>
  </ul>
  
  <h3>Reverse Linked List</h3>
  <ul>
    <li><a href="#">LinkedList - 100</a></li>
    <li><a href="#">LinkedList - 1000</a></li>
  </ul>
</nav>

<main>
  <header>
    <h1>Two Sum · Array · N=1000</h1>
    <p>Two-pointer hash-based solution · Benchmarked 2026-05-27</p>
  </header>
  
  <!-- Primary speedup chart -->
  <div class="section">
    <h2>Speedup vs Input Size</h2>
    <div class="chart-wrap">
      <svg viewBox="0 0 600 250" preserveAspectRatio="xMidYMid meet">
        <!-- Grid -->
        <g stroke="#2a2a4a" stroke-width="1">
          <line x1="60" y1="30" x2="60" y2="220"/>
          <line x1="60" y1="220" x2="580" y2="220"/>
          <line x1="60" y1="175" x2="580" y2="175"/>
          <line x1="60" y1="130" x2="580" y2="130"/>
          <line x1="60" y1="85" x2="580" y2="85"/>
          <line x1="60" y1="40" x2="580" y2="40"/>
        </g>
        
        <!-- Y-axis labels -->
        <text x="55" y="225" text-anchor="end" fill="#8892a0" font-size="11">0</text>
        <text x="55" y="175" text-anchor="end" fill="#8892a0" font-size="11">2x</text>
        <text x="55" y="130" text-anchor="end" fill="#8892a0" font-size="11">4x</text>
        <text x="55" y="85" text-anchor="end" fill="#8892a0" font-size="11">6x</text>
        <text x="55" y="40" text-anchor="end" fill="#8892a0" font-size="11">8x</text>
        
        <!-- Reference line at y=1 -->
        <line class="ref-line" x1="60" y1="188" x2="580" y2="188"/>
        <text x="565" y="183" fill="#8892a0" font-size="10">1x (baseline)</text>
        
        <!-- X-axis labels -->
        <text x="140" y="240" text-anchor="middle" fill="#8892a0" font-size="11">100</text>
        <text x="290" y="240" text-anchor="middle" fill="#8892a0" font-size="11">1K</text>
        <text x="440" y="240" text-anchor="middle" fill="#8892a0" font-size="11">10K</text>
        <text x="560" y="240" text-anchor="middle" fill="#8892a0" font-size="11">100K</text>
        
        <!-- Data line with shading -->
        <path d="M 140 140 L 290 115 L 440 85 L 560 70" 
 fill="none" stroke="#00d9ff" stroke-width="2.5"/>
        
        <!-- Data points -->
        <circle cx="140" cy="140" r="5" fill="#00d9ff"/>
        <circle cx="290" cy="115" r="5" fill="#00d9ff"/>
        <circle cx="440" cy="85" r="5" fill="#00d9ff"/>
        <circle cx="560" cy="70" r="5" fill="#00d9ff"/>
        
        <!-- Value labels -->
        <text x="140" y="132" text-anchor="middle" fill="#00d9ff" font-size="10">3.2x</text>
        <text x="290" y="107" text-anchor="middle" fill="#00d9ff" font-size="10">4.4x</text>
        <text x="440" y="77" text-anchor="middle" fill="#00d9ff" font-size="10">5.7x</text>
        <text x="560" y="62" text-anchor="middle" for="#00d9ff" font-size="10">6.3x</text>
        
        <!-- Axis labels -->
        <text x="320" y="260" text-anchor="middle" fill="#8892a0" font-size="11">Input Size (N)</text>
        <text transform="rotate(-90, 15, 125)" x="15" y="125" text-anchor="middle" fill="#8892a0" font-size="11">Speedup (Python/Sifr)</text>
      </svg>
    </div>
    <div class="legend">
      <div class="legend-item">
        <div class="legend-dot" style="background: var(--sifr)"></div>
        <span>Two Sum (Python/Sifr)</span>
      </div>
      <div class="legend-item">
        <div class="legend-dot" style="background: var(--muted); opacity: 0.5"></div>
        <span>1x baseline (equal performance)</span>
      </div>
    </div>
  </div>
  
  <!-- Absolute runtime overlay -->
  <div class="section">
    <h2>Absolute Runtime (Log Scale)</h2>
    <div class="chart-wrap">
      <svg viewBox="0 0 600 250" preserveAspectRatio="xMidYMid meet">
        <g stroke="#222" stroke-width="1">
          <line x1="60" y1="30" x2="60" y2="220"/>
          <line x1="60" y1="220" x2="580" y2="220"/>
        </g>
        
        <!-- Python line (higher) -->
        <path d="M 140 150 L 290 120 L 440 90 L 560 65" 
              fill="none" stroke="#ffd700" stroke-width="2"/>
        <circle cx="140" cy="150" r="4" fill="#ffd700"/>
        <circle cx="290" cy="120" r="4" fill="#ffd700"/>
        <circle cx="440" cy="90" r="4" fill="#ffd700"/>
        <circle cx="560" cy="65" r="4" fill="#ffd700"/>
        
        <!-- Sifr line (lower) -->
        <path d="M 140 192 L 290 193 L 440 190 L 560 188" 
              fill="none" stroke="#00d9ff" stroke-width="2"/>
        <circle cx="140" cy="192" r="4" fill="#00d9ff"/>
        <circle cx="290" cy="193" r="4" fill="#00d9ff"/>
        <circle cx="440" cy="190" r="4" fill="#00d9ff"/>
        <circle cx="560" cy="188" r="4" fill="#00d9ff"/>
        
        <!-- Axis labels -->
        <text x="140" y="240" text-anchor="middle" fill="#8892a0" font-size="11">100</text>
        <text x="290" y="240" text-anchor="middle" fill="#8892a0" font-size="11">1K</text>
        <text x="440" y="240" text-anchor="middle" fill="#8892a0" font-size="11">10K</text>
        <text x="560" y="240" text-anchor="middle" fill="#8892a0" font-size="11">100K</text>
        
        <text x="320" y="260" text-anchor="middle" fill="#8892a0" font-size="11">Input Size (N)</text>
        <text transform="rotate(-90, 15, 125)" x="15" y="125" text-anchor="middle" fill="#8892a0" font-size="11">Runtime (ms, log)</text>
      </svg>
    </div>
    <div class="legend">
      <div class="legend-item">
        <div class="legend-dot" style="background: #ffd700"></div>
        <span>Python</span>
      </div>
      <div class="legend-item">
        <div class="legend-dot" style="background: #00d9ff"></div>
        <span>Sifr</span>
      </div>
    </div>
  </div>
  
  <!-- Metrics table -->
  <div class="section">
    <h2>Detailed Metrics</h2>
    <table>
      <thead>
        <tr>
          <th>Implementation</th>
          <th class="numeric">Mean (ms)</th>
          <th class="numeric">Median (ms)</th>
          <th class="numeric">Std Dev</th>
          <th class="numeric">CV</th>
          <th class="numeric">Speedup</th>
          <th class="numeric">Mem (MB)</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>Python</td>
          <td class="numeric">4.42</td>
          <td class="numeric">4.38</td>
          <td class="numeric">0.18</td>
          <td class="numeric">4.1%</td>
          <td class="numeric">—</td>
          <td class="numeric">12.4</td>
        </tr>
        <tr>
          <td>Sifr</td>
          <td class="numeric">1.00</td>
          <td class="numeric">0.99</td>
          <td class="numeric">0.02</td>
          <td class="numeric">2.0%</td>
          <td class="speedup wins">4.4x</td>
          <td class="numeric">3.2</td>
        </tr>
      </tbody>
    </table>
  </div>
</main>
</body>
</html>
```

### Key Design Decisions in This Implementation

1. **Dark theme**: Reduces eye strain when comparing many charts; more professional for developer tooling
2. **Log-scale Y-axis for absolute chart**: Required when magnitudes differ by orders of magnitude (Python often 10-100x slower)
3. **Reference line at 1x**: Immediately shows where Sifr wins/loses
4. **No legends inside chart**: Legends belong below the chart; avoid cluttering the data area
5. **Minimal axes**: Only essential gridlines; let the eye focus on data6. **Data labels on key points**: Don't make users guess values; show them near data points

### Scaling to Many Problems

1. **Heatmap in HTML/CSS grid** for category-level overview2. **Lazy-load charts** as user navigates to problems (keep initial page light)
3. **Embed SVG via JavaScript** generation from JSON data—keeps HTML DRY and lets you regenerate charts when benchmark data updates
4. **Collapse categories by default**: Nobody wants 50 charts visible at once

### Data Format for JavaScript

```javascript
const benchmarkData = {
  "two-sum": {
    "category": "Arrays",
    "problems": {
      "N=100": { python: {...}, sifr: {...}, speedup: 3.2 },
      "N=1000": { python: {...}, sifr: {...}, speedup: 4.4 },
      "N=10000": { python: {...}, sifr: {...}, speedup: 5.7 },
    }
  }
};
```

Chart generation becomes a simple function: `drawSpeedupChart(problemData, container)`.
