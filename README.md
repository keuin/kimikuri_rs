# Kimikuri: a message push bot

Kimikuri is a telegram bot which provides every Telegram user a unique token, and an HTTP API which enables them to push
messages to their Telegram. Originally this project was created to replace *ServerChan (方糖)*, which is mainly restricted
by WeChat's poor functionality.

# Usage

1. Start a conversation with [Kimikuri_yourbot](https://t.me/Kimikuri_yourbot).
2. Send a `/start` to obtain a token.
3. Push messages to your Telegram using our HTTP API.

## HTTP API

### Using POST (Recommended)

- Url: `https://kimikuri.keuin.cc/api/message`
- Method: `POST`
- Content-Type: `application/json`
- Request Body:
    ```json
    {
      "token": "<your token>",
      "message": "<message>"
    }
    ```

### Using GET (Depreciated, kept for backward compatibility)

- Url: `https://kimikuri.keuin.cc/api/message`
- Method: `GET`
- Url Parameters:
    + `token`: Your token
    + `message`: Text message

Note: To prevent potential cache on the network, add a nonce parameter (such as a timestamp) if necessary.

## API Example

Here is an example in Python, using `requests`

```python
import requests


def send_message(token: str, message: str):
    r = requests.post(
        'https://kimikuri.keuin.cc/api/message',
        json={
            'token': token,
            'message': message,
        }
    )
    return r.ok and r.json().get('success')


if __name__ == '__main__':
    send_message('<your token>', 'Hello, world!')
```