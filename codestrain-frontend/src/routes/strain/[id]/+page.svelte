<script lang="ts">
	const { data } = $props();

	let fileSelection: FileList | null = $state(null);
	let wasmBase64: string | null = $state(null);
	$effect(() => {
		console.log('files', fileSelection);
		if (fileSelection != null && fileSelection.length > 0) {
			file2Base64(fileSelection[0]).then((base64) => {
				wasmBase64 = base64;
				console.log('wasmBase64', wasmBase64);
			});
		} else {
			wasmBase64 = null;
		}
	});

	const file2Base64 = (file: File): Promise<string> => {
		return new Promise<string>((resolve, reject) => {
			const reader = new FileReader();
			reader.readAsDataURL(file);
			reader.onload = () => resolve(reader.result?.toString().split(',').at(-1) || '');
			reader.onerror = (error) => reject(error);
		});
	};
</script>

<h1>Strain "{data.strain.name}"</h1>
<h2>Available Versions</h2>
{#each data.strain.versions as version, index}
	<p>
		<a href="/strain/{data.strain.id}?version={version.id}">{version.created_at}</a>
		{#if index === 0}
			(latest)
		{/if}
	</p>
{/each}
{#if data.strain.versions.length === 0}
	<p>This strain has no versions yet.</p>
{/if}
{#if data.user && data.user.id === data.strain.creator_id}
	<a href="/strain/{data.strain.id}">Create a new version</a>
{/if}
{#if data.selected_version != null}
	<h2>Version {data.selected_version.id}</h2>
	<p>Created {data.selected_version.created_at}</p>
	<textarea
		name="code"
		placeholder="Code"
		rows="10"
		cols="80"
		value={data.selected_version.code}
		disabled
	/>
{:else if data.user && data.user.id === data.strain.creator_id}
	<h2>Create a new version</h2>
	<input type="file" name="wasm" accept=".wasm" bind:files={fileSelection} />
	{#if wasmBase64}
		<p>WASM file selected.</p>
		{#if fileSelection != null && fileSelection.length > 1}
			<p>Note: You selected multiple files, but only the first one will be submitted!</p>
		{/if}
	{/if}
	{#if data.user && data.user.id === data.strain.creator_id}
		<form method="POST" action="?/create_version">
			<textarea name="code" placeholder="Code" rows="10" cols="80" required></textarea>
			<input type="hidden" name="wasm" value={wasmBase64} required />
			<br />
			<input type="submit" value="Create a new version" />
		</form>
	{/if}
{/if}
