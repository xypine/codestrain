<script lang="ts">
	const { data } = $props();
	const board_size = data.battle.arena_size;
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
	for (const [index, move] of data.battle.log.entries()) {
		const player = index % 2 === 0 ? 'a' : 'b';
		board[move.y][move.x] = {
			player,
			turn: index + 1
		};
	}
	let turn = $state(data.battle.log.length);
</script>

<main>
	<h1>Battle – {data.battle.strain_a} vs. {data.battle.strain_b}</h1>
	<h2>Results</h2>
	{#if data.battle.winner}
		<h3>Winner: {data.battle.winner}</h3>
	{:else}
		<h3>Draw</h3>
	{/if}
	<h3>Score:</h3>
	<p>a: {data.battle.score_a}</p>
	<p>b: {data.battle.score_b}</p>
	<h3>Moves</h3>
	<div class="grid">
		{#each board as row}
			<div class="row">
				{#each row as cell}
					<div class="cell">
						{#if cell && cell.turn <= turn}
							{cell.player}
						{/if}
					</div>
				{/each}
			</div>
		{/each}
	</div>
	<input type="range" min="0" max={data.battle.log.length} bind:value={turn} />
	<p>Turn {turn}</p>
	<p>{data.battle.log.length} moves in total</p>
	{#each data.battle.log as move}
		<p>{JSON.stringify(move)}</p>
	{/each}
</main>

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
		border: 1px solid black;

		display: flex;
		justify-content: center;
		align-items: center;
	}
</style>
