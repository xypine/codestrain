<script lang="ts">
	import Board from '$lib/components/board.svelte';

	const { data } = $props();
	let strain_a_name = $state(data.strain_a.name);
	let strain_b_name = $state(data.strain_b.name);

	const board_size = data.battle.arena_size;
	const log = data.battle.log;
	let autoplay = $state(false);
</script>

<main>
	<h2>
		<span class="a">{strain_a_name}</span> vs. <span class="b">{strain_b_name}</span>
	</h2>
	<h2>Results</h2>
	{#if data.battle.winner}
		<h3>
			Winner:
			{#if data.battle.winner === data.battle.strain_a}
				<span class="a">{strain_a_name}</span>
			{:else}
				<span class="b">{strain_b_name}</span>
			{/if}
		</h3>
	{:else}
		<h3>Draw</h3>
	{/if}
	<h3>Score:</h3>
	<p>a: {data.battle.score_a}</p>
	<p>b: {data.battle.score_b}</p>
	<h3>Moves</h3>
	<Board size={board_size} battle_log={log} {autoplay} singleplayer={false} />
	<label>
		<input type="checkbox" bind:checked={autoplay} />
		autoplay
	</label>
	<!--
	{#each data.battle.log as move}
		<p>{JSON.stringify(move)}</p>
	{/each}-->
</main>

<style>
	.a {
		color: #ff0000;
		background-color: #ff00000f;
	}
	.b {
		color: #0000ff;
		background-color: #0000ff0f;
	}
</style>
