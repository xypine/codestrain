<script lang="ts">
	let { data } = $props();
</script>

<h2>Users</h2>
<ul>
	{#each data.users as user}
		<li>{user.id}</li>
		<ul>
			<li>{user.name}</li>
			<li>{user.email}</li>
			<li>{user.created_at}</li>
			<li>{user.updated_at}</li>
		</ul>
	{/each}
</ul>
