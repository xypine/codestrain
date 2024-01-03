<script lang="ts">
	import type { StrainInput, StrainOutput } from '$lib/common.js';
	import Board from '$lib/components/board.svelte';
	import createPlugin, { type Manifest } from '@extism/extism';

	const { data } = $props();

	const board_size = 10;

	let strain_name: string = $state('');

	async function simulatePlugin(base64: string) {
		const arrayBuffer = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
		const module = await WebAssembly.compile(arrayBuffer);
		const plugin = await createPlugin(module, {
			useWasi: true
		});
		console.log('plugin initialized', plugin);
		let board: (boolean | null)[][] = new Array(board_size)
			.fill(null)
			.map(() => new Array(board_size).fill(null));
		board[0][0] = true;
		let log = [];
		while (true) {
			const empty = board
				.map((row, y) => row.map((cell, x) => ({ x, y, cell })))
				.flat()
				.filter((cell) => cell.cell === null);
			const occupied = board
				.map((row, y) => row.map((cell, x) => ({ x, y, cell })))
				.flat()
				.filter((cell) => cell.cell !== null);
			// Only allow moves that are directly adjacent to an occupied square (no diagonals)
			const allowed = empty.filter((cell) =>
				occupied.some(
					(occupied) =>
						(occupied.x === cell.x && Math.abs(occupied.y - cell.y) === 1) ||
						(occupied.y === cell.y && Math.abs(occupied.x - cell.x) === 1)
				)
			);
			if (allowed.length === 0) {
				break;
			}
			let inputBoard: StrainInput['board'] = [];
			for (let y = 0; y < board.length; y++) {
				for (let x = 0; x < board[y].length; x++) {
					inputBoard.push([[x, y], board[y][x]]);
				}
			}
			const input: StrainInput = {
				board: inputBoard,
				allowed: allowed.map((cell) => [cell.x, cell.y])
			};
			const result = await plugin.call('take_turn', JSON.stringify(input));
			if (!result) {
				throw new Error('Plugin returned null');
			}
			const parsed: StrainOutput = JSON.parse(new TextDecoder().decode(result.buffer));
			console.log('turn', parsed);
			const [x, y] = parsed;
			board[y][x] = true;
			log.push({ x, y });
		}
		return log;
	}
	let simulationResult: { x: number; y: number }[] | null | { error: string } = $state(null);

	let fileSelection: FileList | null = $state(null);
	let wasmBase64: string | null = $state(null);
	$effect(() => {
		console.log('files', fileSelection);
		if (fileSelection != null && fileSelection.length > 0) {
			simulationResult = null;
			if (strain_name === '') {
				strain_name = fileSelection[0].name.replace(/\.wasm$/, '');
			}
			file2Base64(fileSelection[0])
				.then((base64) => {
					wasmBase64 = base64;
					console.log('wasmBase64', wasmBase64);
					simulatePlugin(wasmBase64)
						.then((result) => {
							simulationResult = result;
						})
						.catch((error) => {
							simulationResult = { error: error };
						});
				})
				.catch((error) => {
					simulationResult = { error: error };
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
			<input type="text" name="name" placeholder="Strain name" bind:value={strain_name} />
		</label>
		<br />
		<label>
			WASM executable
			<input type="file" accept=".wasm" bind:files={fileSelection} />
		</label>
		{#if wasmBase64}
			{#if fileSelection != null && fileSelection.length > 1}
				<p>Note: You selected multiple files, but only the first one will be submitted!</p>
			{/if}

			{#if simulationResult === null}
				<p>Simulating...</p>
			{:else if 'error' in simulationResult}
				<p>Simulation failed.</p>
				<p>Error: {simulationResult.error}</p>
			{:else if simulationResult !== null}
				<p>Simulation finished.</p>
				<Board
					size={board_size}
					battle_log={simulationResult}
					autoplay={true}
					singleplayer={true}
				/>
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
