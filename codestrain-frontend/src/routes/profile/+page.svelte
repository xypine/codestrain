<script lang="ts">
	const { data } = $props();
</script>

<h1>Profile</h1>

{#if data.user}
	<p>
		You are logged in as {data.user.name} ({data.user.email})
	</p>
	<a href="/logout">Logout</a>
	<h2>Your Strains</h2>
	{#each data.user.strains as strain}
		<div>
			<p style="display: inline;">
				<a href="/strain/{strain.id}">{strain.name}</a>
			</p>
			<form action="/strain/{strain.id}?/delete" method="POST" style="display: inline;">
				<input type="submit" value="Delete" />
			</form>
		</div>
	{/each}
	{#if data.user.strains.length === 0}
		<p>You have no strains yet.</p>
	{/if}
	<form action="/strain" method="POST">
		<input type="text" name="name" placeholder="Strain name" />
		<input type="submit" value="Create a new strain" />
	</form>
{:else}
	<p>You are not logged in.</p>
{/if}
