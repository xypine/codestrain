<script lang="ts">
	const { data } = $props();
</script>

<h1>Battles</h1>
<h2>Existing</h2>
{#each data.battles as battle}
	<p>
		<a href="/battle/{battle.id}">{battle.strain_a} vs. {battle.strain_b}</a>
	</p>
{/each}
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
