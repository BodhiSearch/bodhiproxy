import pytest
import requests
from bodhiproxy import Server, InvalidServerState


def test_server_start():
  server = Server()
  assert server.status == "running"
  server.stop()
  assert server.status == "stopped"
  with pytest.raises(InvalidServerState) as e:
    server.stop()
  assert str(e.value) == "Server is not running"


def test_server_ping():
  server = Server()
  response = requests.get("http://localhost:3000/ping")
  assert response.status_code == 200
  assert response.text == "pong"
  server.stop()


def test_server_destructor():
  server = Server()
  del server
  with pytest.raises(Exception) as e:
    requests.get("http://localhost:3000/ping")
  assert "Failed to establish a new connection" in str(e.value)


def test_server_on_port():
  port = 8080
  server = Server(port)
  response = requests.get(f"http://localhost:{port}/ping")
  assert response.status_code == 200
  assert response.text == "pong"
  server.stop()
