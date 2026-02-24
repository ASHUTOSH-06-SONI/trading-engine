import { useMemo } from 'react';

export default function KpiCards({ data }) {
    const stats = useMemo(() => {
        if (!data || data.length === 0) return null;

        const best = data[0]; // Already sorted by net_pnl desc
        const avgPnl = data.reduce((s, r) => s + r.net_pnl, 0) / data.length;
        const avgWinRate = data.reduce((s, r) => s + r.win_rate, 0) / data.length;

        return {
            bestPnl: best.net_pnl,
            bestSharpe: best.sharpe_ratio,
            bestDrawdown: best.max_drawdown,
            bestWinRate: best.win_rate,
            totalCombos: data.length,
            avgPnl,
            avgWinRate,
            bestTrades: best.total_trades,
        };
    }, [data]);

    if (!stats) return null;

    return (
        <div className="kpi-grid">
            <div className="kpi-card kpi-card--pnl">
                <div className="kpi-card__icon">💰</div>
                <div className="kpi-card__label">Best Net PnL</div>
                <div className={`kpi-card__value ${stats.bestPnl >= 0 ? 'kpi-card__value--positive' : 'kpi-card__value--negative'}`}>
                    {stats.bestPnl >= 0 ? '+' : ''}{stats.bestPnl.toFixed(2)}
                </div>
                <div className="kpi-card__detail">
                    avg: ${stats.avgPnl.toFixed(2)} across {stats.totalCombos} combos
                </div>
            </div>

            <div className="kpi-card kpi-card--sharpe">
                <div className="kpi-card__icon">📈</div>
                <div className="kpi-card__label">Best Sharpe Ratio</div>
                <div className={`kpi-card__value ${stats.bestSharpe >= 0 ? 'kpi-card__value--blue' : 'kpi-card__value--negative'}`}>
                    {stats.bestSharpe >= 0 ? '+' : ''}{stats.bestSharpe.toFixed(4)}
                </div>
                <div className="kpi-card__detail">
                    {stats.bestTrades} trades in best run
                </div>
            </div>

            <div className="kpi-card kpi-card--drawdown">
                <div className="kpi-card__icon">📉</div>
                <div className="kpi-card__label">Min Max Drawdown</div>
                <div className="kpi-card__value kpi-card__value--orange">
                    {(stats.bestDrawdown * 100).toFixed(3)}%
                </div>
                <div className="kpi-card__detail">
                    lowest drawdown in best result
                </div>
            </div>

            <div className="kpi-card kpi-card--winrate">
                <div className="kpi-card__icon">🎯</div>
                <div className="kpi-card__label">Best Win Rate</div>
                <div className="kpi-card__value kpi-card__value--purple">
                    {(stats.bestWinRate * 100).toFixed(2)}%
                </div>
                <div className="kpi-card__detail">
                    avg: {(stats.avgWinRate * 100).toFixed(2)}% across all combos
                </div>
            </div>
        </div>
    );
}
