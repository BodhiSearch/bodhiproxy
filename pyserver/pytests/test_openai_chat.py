import openai
import pytest
from bodhiproxy import Server


@pytest.fixture
def bodhiproxy():
  server = Server()
  server.start()
  return server


@pytest.fixture
def openai_client():
  client = openai.OpenAI()
  return client


def test_openai_chat():
  ...
