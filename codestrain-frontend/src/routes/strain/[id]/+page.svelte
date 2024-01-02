<script lang="ts">
	const { data } = $props();
</script>

<h1>Strain "{data.strain.name}"</h1>

<p>Created {data.strain.created_at}</p>
<p>Modified {data.strain.updated_at}</p>
<p>
	Creator:
	<a href="/user/{data.strain.creator_id}">{data.strain.creator_id}</a>
	{#if data.user && data.user.id === data.strain.creator_id}
		(you)
	{/if}
</p>
<p>Wasm size: {data.strain.wasm_size}</p>
<p>Wasm hash: {data.strain.wasm_hash}</p>
<p>Code:</p>
<textarea name="code" placeholder="Code" rows="10" cols="80" value={data.strain.code} disabled />
