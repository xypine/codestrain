<script lang="ts">
	const { form } = $props();
</script>

<h1>Login</h1>
{#if form?.message}
	<p class:error={form.error}>{form?.message}</p>
{/if}
<form method="post">
	<label>
		Email
		<input type="email" name="email" />
	</label>
	<label>
		Password
		<input type="password" name="password" />
	</label>
	<button type="submit">Login</button>
</form>

<style>
	.error {
		color: red;
	}
</style>
