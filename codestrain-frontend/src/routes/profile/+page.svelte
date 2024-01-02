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
	<h2>Create a new strain</h2>
	<form method="POST" action="/strain">
		<label>
			Name
			<input type="text" name="name" placeholder="Strain name" />
		</label>
		<br />
		<label>
			WASM executable
			<input type="file" accept=".wasm" bind:files={fileSelection} />
		</label>
		{#if wasmBase64}
			<p>WASM file selected.</p>
			{#if fileSelection != null && fileSelection.length > 1}
				<p>Note: You selected multiple files, but only the first one will be submitted!</p>
			{/if}
		{/if}
		<br />
		<label>
			Description <br />
			<textarea name="description" placeholder="Strain description" rows="10" cols="80"></textarea>
		</label>
		<br />
		<label>
			Code <br />
			<textarea name="code" placeholder="Code" rows="10" cols="80" required></textarea>
		</label>
		<input type="hidden" name="wasm" value={wasmBase64} required />
		<br />

		<input type="submit" value="Create a new strain" />
	</form>
{:else}
	<p>You are not logged in.</p>
{/if}
