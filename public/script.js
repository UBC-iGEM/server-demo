const ws = new WebSocket("ws://127.0.0.1:9999/ws");
ws.onmessage = (event) => {
  console.log("Received:", event.data);
};
