<script lang="ts">
	let name = 'John Doe';
	let connected = false;
	let socket = null;
	let messages = [];
	let message = '';

	function connect() {
		socket = new WebSocket('ws://192.168.68.115:3030/socket/' + name);
		socket.onopen = () => {
			socket.send(
				JSON.stringify({
					action: 'auth_request',
					data: {
						app_key: "wR@aF#EVP%42Eh"
					}
				})
			)
			console.log('connected');
			connected = true;
		};

		socket.onclose = () => {
			console.log('disconnected');
			connected = false;
		};

		socket.onmessage = (event) => {
			console.log('message', event.data);
			messages = [...messages, event.data];
		};
	}

	function send() {
		socket.send(message);
		messages = [...messages, message];
		message = '';
	}
</script>

<main>
	{#if connected}
		<h1>Connected</h1>

		<ul>
			{#each messages as message}
				<li>{message}</li>
			{/each}
		</ul>

		<input bind:value={message}/>
		<button on:click={send}>Send</button>
	{:else}
		<h1>Not connected</h1>
		<input bind:value={name}/>
		<button on:click={connect}>Connect</button>
	{/if}


</main>

<style>
</style>
