<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { request_battle } from '$lib/backend.js';

	const { data } = $props();

	const strains_with_score = data.strains
		.map((s) => {
			const scores = data.battles.map((battle) => {
				if (battle.strain_a === s.id) {
					return battle.score_a;
				} else if (battle.strain_b === s.id) {
					return battle.score_b;
				}
				return null;
			});
			const score = scores.map((s) => s ?? 0).reduce((a, b) => a + b, 0);
			const avg_score = score / scores.filter((s) => s !== null).length;
			return { ...s, score, avg_score };
		})
		.sort((a, b) => {
			if (a.score > b.score) return -1;
			if (a.score < b.score) return 1;
			return 0;
		});

	const target_arena_size = 29;
	const missing_combos = $state(
		new Set(
			data.strains
				.flatMap((a) => data.strains.map((b) => [a, b]))
				.filter(
					([a, b]) =>
						!data.battles.some(
							(battle) =>
								battle.arena_size == target_arena_size &&
								battle.strain_a === a.id &&
								battle.strain_b === b.id
						)
				)
				.filter(
					([a, b]) =>
						!data.battles.some(
							(battle) =>
								battle.arena_size == target_arena_size &&
								battle.strain_a === b.id &&
								battle.strain_b === a.id
						)
				)
		)
	);
	async function request_missing() {
		await Promise.all(
			[...missing_combos].map(async ([a, b]) => {
				await request_battle(a.id, b.id, data.session);
				invalidateAll();
			})
		);
		window.location.reload();
	}
</script>

<h1>Battles</h1>
<h2>Existing ({data.battles.length})</h2>

<!-- Now in 2d -->
<table>
	<thead>
		<tr>
			<th>Strain</th>
			{#each data.strains as strain}
				<th><a class="a" href={`/strain/${strain.id}`}>{strain.name}</a></th>
			{/each}
		</tr>
	</thead>
	<tbody>
		{#each data.strains as strain_b}
			<tr>
				<th><a class="b" href={`/strain/${strain_b.id}`}>{strain_b.name}</a></th>
				{#each data.strains as strain_a}
					{@const battle = data.battles.find(
						(battle) => battle.strain_a === strain_a.id && battle.strain_b === strain_b.id
					)}
					{@const color =
						battle?.winner === strain_a.id ? 'a' : battle?.winner === strain_b.id ? 'b' : 'draw'}
					<td>
						{#if battle}
							<a class={color} href={`/battle/${battle.id}`}> open </a>
						{:else}
							-
						{/if}
					</td>
				{/each}
			</tr>
		{/each}
	</tbody>
</table>
{#if data.battles.length === 0}
	<p>No battles yet.</p>
{/if}
<h2>Leaderboard</h2>
<table>
	<thead>
		<tr>
			<th>Strain</th>
			<th>Score</th>
			<th>Avg.</th>
		</tr>
	</thead>
	<tbody>
		{#each strains_with_score as strain}
			<tr>
				<td><a href={`/strain/${strain.id}`}>{strain.name}</a></td>
				<td>{strain.score}</td>
				<td
					class={strain.avg_score > (target_arena_size * target_arena_size) / 2
						? 'a'
						: strain.avg_score < (target_arena_size * target_arena_size) / 2
							? 'b'
							: 'draw'}>{strain.avg_score.toFixed(2)}</td
				>
			</tr>
		{/each}
	</tbody>
</table>
<h2>Request a new battle</h2>
{#if data.user?.admin}
	{#if data.strains.length > 0}
		<form method="POST">
			<select name="strain_a">
				{#each data.strains as strain}
					<option value={strain.id}>{strain.name}</option>
				{/each}
			</select>
			<select name="strain_b">
				{#each data.strains as strain}
					<option value={strain.id}>{strain.name}</option>
				{/each}
			</select>
			<button type="submit">Battle</button>
		</form>
	{:else}
		<p>No strains yet. Create a strain first.</p>
	{/if}
	<p>
		<button on:click={request_missing}>Request all missing</button>
	</p>
{:else}
	<p>If you want to request a new battle, please contact an admin.</p>
{/if}
{#if missing_combos.size > 0}
	<h2>Not yet requested ({missing_combos.size})</h2>
	<table>
		<thead>
			<tr>
				<th>Strain A</th>
				<th>Strain B</th>
			</tr>
		</thead>
		<tbody>
			{#each [...missing_combos] as [a, b]}
				<tr>
					<td><a class="a" href={`/strain/${a.id}`}>{a.name}</a></td>
					<td><a class="b" href={`/strain/${b.id}`}>{b.name}</a></td>
				</tr>
			{/each}
		</tbody>
	</table>
{:else}
	<p>All possible battles played</p>
{/if}

<style>
	a {
		text-decoration: none;
	}
	.a {
		color: #ff0000;
		background-color: #ff00000f;
	}
	.b {
		color: #0000ff;
		background-color: #0000ff0f;
	}
	.draw {
		color: #000000;
		background-color: #0000000f;
	}
</style>
