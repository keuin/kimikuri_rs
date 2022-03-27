import json
import sqlite3


# noinspection SqlNoDataSourceInspection
def convert(db_file: str = 'kimikuri.db', json_file: str = 'users.json'):
    db = sqlite3.connect(db_file)
    with db, open(json_file, 'r', encoding='utf-8') as f:
        j = json.load(f)
        cur = db.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS "user" (
                "id"	INTEGER NOT NULL CHECK(id >= 0) UNIQUE,
                "name"	TEXT,
                "token"	TEXT NOT NULL UNIQUE,
                "chat_id"	INTEGER NOT NULL CHECK(chat_id >= 0) UNIQUE,
                PRIMARY KEY("id","token","chat_id")
        )''')
        cur.executemany(
            'INSERT INTO user (id, name, token, chat_id) VALUES (?,?,?,?)',
            [(x['user_id'], None, x['token'], x['chat_id']) for x in j]
        )
    db.close()


if __name__ == '__main__':
    convert()
