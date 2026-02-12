import numpy as np


def print_report(results: dict) -> None:
    """Print a formatted summary of backtest results."""
    print("=" * 60)
    print("  BACKTEST RESULTS")
    print("=" * 60)
    print(f"  Total P&L:            ${results['total_pnl']:>12,.2f}")
    print(f"  Number of Trades:      {results['num_trades']:>12,}")
    print(f"    Long:                {results['num_long']:>12,}")
    print(f"    Short:               {results['num_short']:>12,}")
    print(f"  Wins / Losses:         {results['num_wins']:>6,} / {results['num_losses']:,}")
    print(f"  Win Rate:              {results['win_rate']:>11.1%}")
    print(f"  Profit Factor:         {results['profit_factor']:>12.2f}")
    print("-" * 60)
    print(f"  Avg Win:              ${results['avg_win']:>12,.2f}")
    print(f"  Avg Loss:             ${results['avg_loss']:>12,.2f}")
    print(f"  Largest Win:          ${results['largest_win']:>12,.2f}")
    print(f"  Largest Loss:         ${results['largest_loss']:>12,.2f}")
    print("-" * 60)
    print(f"  Max Drawdown:         ${results['max_drawdown']:>12,.2f}")
    print(f"  Max Drawdown %:        {results['max_drawdown_pct']:>11.2f}%")
    print(f"  Sharpe Ratio:          {results['sharpe_ratio']:>12.3f}")
    print(f"  Avg Holding Time:      {results['avg_holding_time_secs']:>10.1f}s")
    print("=" * 60)


def plot_equity(results: dict, title: str = "Equity Curve", save_path: str = None) -> None:
    """Plot equity curve and drawdown chart."""
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    equity = np.array(results["equity_curve"])
    if len(equity) == 0:
        print("No equity data to plot.")
        return

    # Compute drawdown
    peak = np.maximum.accumulate(equity)
    drawdown = peak - equity

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(14, 8), sharex=True,
                                     gridspec_kw={"height_ratios": [3, 1]})

    ax1.plot(equity, linewidth=0.8, color="steelblue")
    ax1.set_title(title)
    ax1.set_ylabel("Equity ($)")
    ax1.grid(True, alpha=0.3)
    ax1.axhline(y=0, color="gray", linestyle="--", linewidth=0.5)

    ax2.fill_between(range(len(drawdown)), drawdown, color="salmon", alpha=0.7)
    ax2.set_ylabel("Drawdown ($)")
    ax2.set_xlabel("Bar Index")
    ax2.grid(True, alpha=0.3)

    plt.tight_layout()
    if save_path:
        plt.savefig(save_path, dpi=150)
        print(f"Saved equity chart to {save_path}")
    else:
        plt.savefig("equity_curve.png", dpi=150)
        print("Saved equity chart to equity_curve.png")
    plt.close()
