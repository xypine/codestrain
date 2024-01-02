<script lang="ts">
	const { data } = $props();
	function strain_name(id: string) {
		return data.strains.find((strain) => strain.id === id)?.name || id;
	}
</script>

<h1>Battles</h1>
<h2>Existing</h2>
<table>
	<thead>
		<tr>
			<th>Strain A</th>
			<th>Strain B</th>
			<th>Winner</th>
		</tr>
	</thead>
	<tbody>
		{#each data.battles as battle}
			<tr>
				<td><a class="a" href={`/strain/${battle.strain_a}`}>{strain_name(battle.strain_a)}</a></td>
				<td><a class="b" href={`/strain/${battle.strain_b}`}>{strain_name(battle.strain_b)}</a></td>
				<td>
					{#if battle.winner === null}
						-
					{:else}
						<span class={battle.winner === battle.strain_a ? 'a' : 'b'}>
							{strain_name(battle.winner)}
						</span>
					{/if}
				</td>
				<td><a href="/battle/{battle.id}">open</a></td>
			</tr>
		{/each}
	</tbody>
</table>
{#if data.battles.length === 0}
	<p>No battles yet.</p>
{/if}
<h2>Request a new battle</h2>
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
</style>
