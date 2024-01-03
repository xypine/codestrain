<script lang="ts">
	import type { BattleResult } from '$lib/backend';

	const { size, battle_log, autoplay, singleplayer } = $props();
	const board_size = size as number;
	const log = battle_log as BattleResult['log'];

	type Cell = null | {
		player: 'a' | 'b';
		turn: number;
	};
	const board: Cell[][] = new Array(board_size)
		.fill(null)
		.map(() => new Array(board_size).fill(null));
	board[0][0] = {
		player: 'a',
		turn: 0
	};
	board[board_size - 1][board_size - 1] = {
		player: 'b',
		turn: 0
	};
	for (const [index, move] of log.entries()) {
		const player = singleplayer ? 'a' : move.player ? 'a' : 'b';
		board[move.y][move.x] = {
			player,
			turn: index + 1
		};
	}
	let turn = $state(autoplay ? 0 : log.length);

	let autoplay_interval: number | null = null;
	$effect(() => {
		if (autoplay) {
			autoplay_interval = setInterval(() => {
				turn = (turn + 1) % (log.length + 1);
			}, 50);
			return () => {
				if (autoplay_interval) {
					clearInterval(autoplay_interval);
				}
			};
		} else {
			if (autoplay_interval) {
				clearInterval(autoplay_interval);
			}
		}
	});
</script>

<div class="grid">
	{#each board as row, y}
		<div class="row">
			{#each row as cell, x}
				<div class={`cell ${cell.turn <= turn && cell?.player}`}>
					{#if cell && cell.turn <= turn}
						{cell.player}
					{/if}
				</div>
			{/each}
		</div>
	{/each}
</div>
<input type="range" min="0" max={log.length} bind:value={turn} />
<p>Turn {turn} / {log.length}</p>

<style>
	.grid {
		display: grid;
		gap: 1px;
	}
	.row {
		display: flex;
		gap: 1px;
	}
	.cell {
		width: 20px;
		height: 20px;
		border: 1px solid #0004;

		display: flex;
		justify-content: center;
		align-items: center;
	}
	.a {
		color: #ff0000;
		background-color: #ff00000f;
	}
	.b {
		color: #0000ff;
		background-color: #0000ff0f;
	}
</style>
