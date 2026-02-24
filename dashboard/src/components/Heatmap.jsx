import { useMemo, useState, useCallback } from 'react';

function pnlToColor(value, min, max) {
    // Normalize to 0-1 range
    const range = max - min || 1;
    const normalized = (value - min) / range;

    // Red → Orange → Green gradient
    if (normalized < 0.5) {
        // Red to Orange
        const t = normalized * 2;
        const r = Math.round(239 + (245 - 239) * t);
        const g = Math.round(68 + (158 - 68) * t);
        const b = Math.round(68 + (11 - 68) * t);
        return `rgba(${r}, ${g}, ${b}, 0.7)`;
    } else {
        // Orange to Green
        const t = (normalized - 0.5) * 2;
        const r = Math.round(245 + (16 - 245) * t);
        const g = Math.round(158 + (185 - 158) * t);
        const b = Math.round(11 + (129 - 11) * t);
        return `rgba(${r}, ${g}, ${b}, 0.7)`;
    }
}

export default function Heatmap({ data }) {
    const [tooltip, setTooltip] = useState(null);

    const { grid, obiValues, cdValues, pnlMin, pnlMax } = useMemo(() => {
        if (!data || data.length === 0) return { grid: {}, obiValues: [], cdValues: [], pnlMin: 0, pnlMax: 0 };

        // For each (obi, cooldown) pick the best target_profit_bps result
        const bestByCell = {};
        let pnlMin = Infinity;
        let pnlMax = -Infinity;

        for (const r of data) {
            const key = `${r.params.obi_threshold.toFixed(2)}_${r.params.cooldown_steps}`;
            if (!bestByCell[key] || r.net_pnl > bestByCell[key].net_pnl) {
                bestByCell[key] = r;
            }
        }

        // Find ranges from aggregated cells
        for (const r of Object.values(bestByCell)) {
            if (r.net_pnl < pnlMin) pnlMin = r.net_pnl;
            if (r.net_pnl > pnlMax) pnlMax = r.net_pnl;
        }

        // Get unique sorted OBI thresholds and cooldowns
        const obiSet = new Set();
        const cdSet = new Set();
        for (const r of data) {
            obiSet.add(r.params.obi_threshold.toFixed(2));
            cdSet.add(r.params.cooldown_steps);
        }

        const obiValues = [...obiSet].sort((a, b) => parseFloat(a) - parseFloat(b));
        const cdValues = [...cdSet].sort((a, b) => a - b);

        return { grid: bestByCell, obiValues, cdValues, pnlMin, pnlMax };
    }, [data]);

    const handleMouseMove = useCallback((e, result) => {
        setTooltip({
            x: e.clientX + 12,
            y: e.clientY - 10,
            result,
        });
    }, []);

    const handleMouseLeave = useCallback(() => {
        setTooltip(null);
    }, []);

    if (obiValues.length === 0) return null;

    return (
        <div className="heatmap-container">
            <div className="section-header">
                <span className="section-header__icon">🗺️</span>
                <h2 className="section-header__title">Parameter Performance Heatmap</h2>
                <span className="section-header__badge">OBI × Cooldown (Best Target BPS)</span>
            </div>

            <div className="heatmap-wrapper">
                <div className="heatmap">
                    {cdValues.map((cd) => (
                        <div className="heatmap__row" key={cd}>
                            <div className="heatmap__y-label">{cd}</div>
                            {obiValues.map((obi) => {
                                const key = `${obi}_${cd}`;
                                const result = grid[key];
                                const pnl = result ? result.net_pnl : null;
                                const color = pnl !== null ? pnlToColor(pnl, pnlMin, pnlMax) : 'rgba(255,255,255,0.03)';

                                return (
                                    <div
                                        key={key}
                                        className="heatmap__cell"
                                        style={{ background: color }}
                                        onMouseMove={(e) => result && handleMouseMove(e, result)}
                                        onMouseLeave={handleMouseLeave}
                                    >
                                        {pnl !== null ? pnl.toFixed(0) : '—'}
                                    </div>
                                );
                            })}
                        </div>
                    ))}

                    <div className="heatmap__x-labels">
                        {obiValues.map((obi) => (
                            <div className="heatmap__x-label" key={obi}>{obi}</div>
                        ))}
                    </div>
                </div>

                <div className="heatmap__axis-label">OBI Threshold →</div>

                <div className="heatmap__legend">
                    <span>Low PnL (Loss)</span>
                    <div className="heatmap__legend-bar" />
                    <span>High PnL (Profit)</span>
                </div>
            </div>

            {tooltip && (
                <div className="heatmap__tooltip" style={{ left: tooltip.x, top: tooltip.y }}>
                    <div>OBI: <strong>{tooltip.result.params.obi_threshold.toFixed(2)}</strong></div>
                    <div>Cooldown: <strong>{tooltip.result.params.cooldown_steps}</strong></div>
                    <div>Target BPS: <strong>{tooltip.result.params.target_profit_bps}</strong></div>
                    <div>Net PnL: <strong style={{ color: tooltip.result.net_pnl >= 0 ? '#10b981' : '#ef4444' }}>
                        ${tooltip.result.net_pnl.toFixed(2)}
                    </strong></div>
                    <div>Sharpe: <strong>{tooltip.result.sharpe_ratio.toFixed(4)}</strong></div>
                    <div>Trades: <strong>{tooltip.result.total_trades}</strong></div>
                </div>
            )}
        </div>
    );
}
