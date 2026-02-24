import { useMemo, useState } from 'react';

const COLUMNS = [
    { key: 'rank', label: '#', sortable: false },
    { key: 'obi_threshold', label: 'OBI Threshold', sortable: true },
    { key: 'cooldown_steps', label: 'Cooldown', sortable: true },
    { key: 'target_profit_bps', label: 'Target BPS', sortable: true },
    { key: 'net_pnl', label: 'Net PnL', sortable: true },
    { key: 'sharpe_ratio', label: 'Sharpe', sortable: true },
    { key: 'max_drawdown', label: 'Max DD', sortable: true },
    { key: 'win_rate', label: 'Win Rate', sortable: true },
    { key: 'total_trades', label: 'Trades', sortable: true },
];

function getValue(result, key) {
    switch (key) {
        case 'obi_threshold': return result.params.obi_threshold;
        case 'cooldown_steps': return result.params.cooldown_steps;
        case 'target_profit_bps': return result.params.target_profit_bps;
        default: return result[key];
    }
}

function RankBadge({ rank }) {
    let cls = 'rank-badge--default';
    if (rank === 1) cls = 'rank-badge--gold';
    else if (rank === 2) cls = 'rank-badge--silver';
    else if (rank === 3) cls = 'rank-badge--bronze';

    return <span className={`rank-badge ${cls}`}>{rank}</span>;
}

export default function ResultsTable({ data }) {
    const [sortKey, setSortKey] = useState('net_pnl');
    const [sortDir, setSortDir] = useState('desc');
    const [filter, setFilter] = useState('');

    const handleSort = (key) => {
        if (!COLUMNS.find(c => c.key === key)?.sortable) return;
        if (sortKey === key) {
            setSortDir(d => d === 'asc' ? 'desc' : 'asc');
        } else {
            setSortKey(key);
            setSortDir('desc');
        }
    };

    const sortedData = useMemo(() => {
        if (!data) return [];

        let filtered = data;
        if (filter) {
            const f = filter.toLowerCase();
            filtered = data.filter(r =>
                r.params.obi_threshold.toFixed(2).includes(f) ||
                r.params.cooldown_steps.toString().includes(f) ||
                r.params.target_profit_bps.toString().includes(f) ||
                r.net_pnl.toFixed(2).includes(f)
            );
        }

        return [...filtered].sort((a, b) => {
            const av = getValue(a, sortKey);
            const bv = getValue(b, sortKey);
            const cmp = av < bv ? -1 : av > bv ? 1 : 0;
            return sortDir === 'asc' ? cmp : -cmp;
        });
    }, [data, sortKey, sortDir, filter]);

    if (!data || data.length === 0) return null;

    return (
        <div className="table-container">
            <div className="table-toolbar">
                <div className="section-header" style={{ marginBottom: 0 }}>
                    <span className="section-header__icon">📋</span>
                    <h2 className="section-header__title">Research Results</h2>
                    <span className="section-header__badge">{sortedData.length} strategies</span>
                </div>
                <input
                    type="text"
                    className="table-search"
                    placeholder="Filter by value..."
                    value={filter}
                    onChange={(e) => setFilter(e.target.value)}
                    id="results-filter"
                />
            </div>

            <div style={{ overflowX: 'auto' }}>
                <table className="results-table">
                    <thead>
                        <tr>
                            {COLUMNS.map((col) => (
                                <th
                                    key={col.key}
                                    onClick={() => handleSort(col.key)}
                                    className={sortKey === col.key ? 'th--active' : ''}
                                >
                                    {col.label}
                                    {col.sortable && (
                                        <span className="sort-arrow">
                                            {sortKey === col.key ? (sortDir === 'asc' ? '▲' : '▼') : '⇅'}
                                        </span>
                                    )}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {sortedData.map((r, i) => {
                            const rank = i + 1;
                            const isTop = rank <= 3;

                            return (
                                <tr key={i} className={isTop ? 'row--top' : ''}>
                                    <td><RankBadge rank={rank} /></td>
                                    <td>{r.params.obi_threshold.toFixed(2)}</td>
                                    <td>{r.params.cooldown_steps}</td>
                                    <td>{r.params.target_profit_bps.toFixed(1)}</td>
                                    <td className={r.net_pnl >= 0 ? 'cell--positive' : 'cell--negative'}>
                                        {r.net_pnl >= 0 ? '+' : ''}{r.net_pnl.toFixed(2)}
                                    </td>
                                    <td className={r.sharpe_ratio >= 0 ? 'cell--positive' : 'cell--negative'}>
                                        {r.sharpe_ratio >= 0 ? '+' : ''}{r.sharpe_ratio.toFixed(4)}
                                    </td>
                                    <td>{(r.max_drawdown * 100).toFixed(3)}%</td>
                                    <td>{(r.win_rate * 100).toFixed(2)}%</td>
                                    <td>{r.total_trades.toLocaleString()}</td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
