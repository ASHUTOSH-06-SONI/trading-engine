import { useState, useEffect } from 'react';
import KpiCards from './components/KpiCards';
import Heatmap from './components/Heatmap';
import ResultsTable from './components/ResultsTable';

export default function App() {
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    fetch('/research_results.json')
      .then((res) => {
        if (!res.ok) throw new Error('Failed to load research_results.json');
        return res.json();
      })
      .then((results) => {
        // Sort by net_pnl descending
        results.sort((a, b) => b.net_pnl - a.net_pnl);
        setData(results);
        setLoading(false);
      })
      .catch((err) => {
        setError(err.message);
        setLoading(false);
      });
  }, []);

  if (loading) {
    return (
      <div className="loading-container">
        <div className="loading-spinner" />
        <div className="loading-text">Loading research results...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="loading-container">
        <div style={{ fontSize: 48 }}>⚠️</div>
        <div className="loading-text">{error}</div>
        <div className="loading-text" style={{ fontSize: 12 }}>
          Run the research agent first: <code>cargo run --release -p launcher -- --mode research --file data.csv</code>
        </div>
      </div>
    );
  }

  return (
    <div className="dashboard">
      <header className="header">
        <span className="header__icon">✨</span>
        <h1 className="header__title">Trading Research Dashboard</h1>
        <p className="header__subtitle">
          Grid Search Results • {data.length} parameter combinations analyzed
        </p>
      </header>

      <KpiCards data={data} />
      <Heatmap data={data} />
      <ResultsTable data={data} />

      <footer className="footer">
        Trading Engine Research Agent • Maker Fee: 0.02% • Taker Fee: 0.05% • Powered by Rayon
      </footer>
    </div>
  );
}
