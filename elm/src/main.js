import './style.css';
import { Elm } from './Main.elm';

// Initialize Elm app
const app = Elm.Main.init({
  node: document.getElementById('app')
});

// WebSocket connection
let ws = null;
let reconnectTimer = null;
let reconnectAttempts = 0;
const MAX_RECONNECT_ATTEMPTS = 5;
const RECONNECT_DELAY = 3000;

function connectWebSocket() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.host}/ws`;

  console.log('Connecting to WebSocket:', wsUrl);

  ws = new WebSocket(wsUrl);

  ws.onopen = () => {
    console.log('WebSocket connected');
    reconnectAttempts = 0;
    if (app.ports.websocketConnected) {
      app.ports.websocketConnected.send(null);
    }
  };

  ws.onmessage = (event) => {
    console.log('WebSocket message received:', event.data);
    if (app.ports.websocketIn) {
      app.ports.websocketIn.send(event.data);
    }
  };

  ws.onerror = (error) => {
    console.error('WebSocket error:', error);
  };

  ws.onclose = () => {
    console.log('WebSocket disconnected');
    if (app.ports.websocketDisconnected) {
      app.ports.websocketDisconnected.send(null);
    }

    // Attempt to reconnect
    if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
      reconnectAttempts++;
      console.log(`Reconnecting in ${RECONNECT_DELAY}ms (attempt ${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`);
      reconnectTimer = setTimeout(connectWebSocket, RECONNECT_DELAY);
    } else {
      console.error('Max reconnection attempts reached');
    }
  };
}

// Send messages to WebSocket
if (app.ports.websocketOut) {
  app.ports.websocketOut.subscribe((message) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(message);
    } else {
      console.warn('WebSocket not open, message not sent:', message);
    }
  });
}

// Start WebSocket connection
connectWebSocket();

// Handle page visibility changes to reconnect if needed
document.addEventListener('visibilitychange', () => {
  if (!document.hidden && (!ws || ws.readyState !== WebSocket.OPEN)) {
    console.log('Page visible, reconnecting WebSocket...');
    connectWebSocket();
  }
});

// Cleanup on page unload
window.addEventListener('beforeunload', () => {
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
  }
  if (ws) {
    ws.close();
  }
});
