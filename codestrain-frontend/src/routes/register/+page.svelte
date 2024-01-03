<script lang="ts">
	const { form } = $props();
</script>

<h1>Register</h1>
{#if form?.message}
	<p class:error={form.error}>{form?.message}</p>
{/if}
<p>
	Note: For now details other than the password are public, so if you don't want to share your email
	address, use a fake one.
</p>
<form method="post">
	<label>
		Name
		<input type="text" name="name" />
	</label>
	<label>
		Email
		<input type="email" name="email" />
	</label>
	<label>
		Password
		<input type="password" name="password" />
	</label>
	<button type="submit">Register</button>
</form>

<style>
	.error {
		color: red;
	}
</style>
