import asyncio
import random

import pytest
import requests
from bodhiproxy import InvalidServerState, Server


@pytest.fixture
def random_port():
  return random.randint(3000, 4000)


@pytest.mark.asyncio
async def test_server_start(random_port):
  server = await Server.start_server(random_port)
  assert server.status == "running"
  await server.stop()
  assert server.status == "stopped"
  with pytest.raises(InvalidServerState) as e:
    server.stop()
  assert str(e.value) == "Server is not running"


@pytest.mark.asyncio
async def test_server_ping(random_port):
  server = await Server.start_server(random_port)
  response = requests.get(f"http://localhost:{random_port}/ping")
  assert response.status_code == 200
  assert response.text == "pong"
  await server.stop()


@pytest.mark.asyncio
async def test_server_stop(random_port):
  server = await Server.start_server(random_port)
  await server.stop()
  with pytest.raises(Exception) as e:
    _ = requests.get(f"http://localhost:{random_port}/ping")
  assert "Failed to establish a new connection" in str(e.value)


@pytest.mark.asyncio
async def test_server_del(random_port):
  server = await Server.start_server(random_port)
  del server
  for _ in range(3):
    try:
      _ = requests.get(f"http://localhost:{random_port}/ping")
      await asyncio.sleep(0.1)
    except requests.exceptions.ConnectionError as e:
      assert "Failed to establish a new connection" in str(e)
      return
  raise AssertionError("the server was not closed when the handle was deleted")


@pytest.mark.asyncio
async def test_server_on_port(random_port):
  server = await Server.start_server(random_port)
  response = requests.get(f"http://localhost:{random_port}/ping")
  assert response.status_code == 200
  assert response.text == "pong"
  server.stop()
