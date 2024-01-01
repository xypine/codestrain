<script lang="ts">
	const { data } = $props();
</script>

<h1>StrainLab</h1>
<nav>
	<ul>
		<li>
			<a href="/">Home</a>
		</li>
		{#if data.session}
			<li>
				<a href="/profile">Profile</a>
			</li>
		{:else}<li>
				<a href="/login">Login</a>
			</li>
			<li>
				<a href="/register">Register</a>
			</li>
		{/if}
	</ul>
</nav>
<slot />
