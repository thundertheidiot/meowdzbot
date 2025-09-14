function playerFilter(p) {
	console.log(p.name);
	return !p.name && p.name !== "DatHost - GOTV";
}

function showServerInfo(div, data) {
	if (data.ServerUp) {
		const sinfo = data.ServerUp.server_info;
		const elapsed = data.ServerUp.elapsed;
		const img = data.ServerUp.image;

		var players = data.ServerUp.players;
		players = players.filter(playerFilter);

		const name = document.createElement("h2");
		name.innerHTML = sinfo.name;

		const map = document.createElement("code");
		map.innerHTML = `${sinfo.map} - ${players.length}/${sinfo.max_players} players online`;

		// const time = document.createElement("code");
		// time.innerHTML = `Time since map change \`${}\``;

		const playerList = document.createElement("code");
		playerList.innerHTML = "";

		const image = document.createElement("img");
		image.style = "width: 90%; border-radius: 5px;";

		if (img != null) {
			image.src = `/static/maps/${img}`;
		}

		div.replaceChildren(
			name,
			map,
			document.createElement("br"),
			playerList,
			document.createElement("br"),
			image,
		);
	} else if (data.ServerDown) {
	} else {
		console.error("Unexpected data format:", data);
	}
}

function fetchServerData(server) {
	fetch(`/data/${server}`)
		.then((response) => response.json())
		.then((data) => {
			const div = document.getElementById(server);

			showServerInfo(div, data);
		})
		.catch((error) => console.error("Error fetching data:", error));
}

fetchServerData("meow");
// fetchServerData("meow2");

const meowTimer = setInterval(fetchServerData, 3000, "meow");
// const meow2Timer = setInterval(fetchServerData, 3000, "meow2");
