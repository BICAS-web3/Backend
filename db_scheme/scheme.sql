DROP TABLE IF EXISTS NativeCurrency CASCADE;
DROP TABLE IF EXISTS Network CASCADE;
DROP TABLE IF EXISTS RpcUrl CASCADE;
DROP TABLE IF EXISTS BlockExplorerUrl CASCADE;
DROP TABLE IF EXISTS Token CASCADE;
DROP TABLE IF EXISTS Game CASCADE;
DROP TABLE IF EXISTS Nickname CASCADE;
DROP TABLE IF EXISTS Player CASCADE;
DROP TABLE IF EXISTS Bet CASCADE;
DROP TABLE IF EXISTS BanWords CASCADE;


CREATE TABLE IF NOT EXISTS NativeCurrency(
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    decimals BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS Network(
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    short_name TEXT NOT NULL,
    native_currency_id BIGSERIAL NOT NULL,

    CONSTRAINT fk_nativecurrency
        FOREIGN KEY(native_currency_id)
            REFERENCES NativeCurrency(id)
);

CREATE VIEW NetworkInfo AS
    SELECT network.id as network_id,
            network.name as network_name,
            network.short_name as short_name,
            nativecurrency.name as currency_name,
            nativecurrency.symbol as currency_symbol,
            nativecurrency.decimals
        FROM Network 
    INNER JOIN NativeCurrency 
        ON Network.native_currency_id = NativeCurrency.id;

CREATE TABLE IF NOT EXISTS RpcUrl(
    id BIGSERIAL PRIMARY KEY,
    network_id BIGSERIAL NOT NULL,
    url TEXT NOT NULL,

    CONSTRAINT fk_network
        FOREIGN KEY(network_id)
            REFERENCES Network(id)
);

CREATE TABLE IF NOT EXISTS BlockExplorerUrl(
    id BIGSERIAL PRIMARY KEY,
    network_id BIGSERIAL NOT NULL,
    url TEXT NOT NULL,

    CONSTRAINT fk_network
        FOREIGN KEY(network_id)
            REFERENCES Network(id)
);

CREATE TABLE IF NOT EXISTS Token(
    id BIGSERIAL PRIMARY KEY,
    network_id BIGSERIAL NOT NULL,
    name TEXT NOT NULL,
    contract_address character(42) NOT NULL,

    CONSTRAINT fk_network
        FOREIGN KEY(network_id)
            REFERENCES Network(id)
);

CREATE UNIQUE INDEX token_network_id_idx ON Token(network_id, contract_address);
CREATE INDEX token_idx ON Token(contract_address);

CREATE TABLE IF NOT EXISTS GameAbi(
    signature character(66) NOT NULL PRIMARY KEY,
    types TEXT NOT NULL,
    names TEXT NOT NULL
);

-- CREATE UNIQUE INDEX game_address_idx ON GameAbi(signature);

CREATE TABLE IF NOT EXISTS Game(
    id BIGSERIAL PRIMARY KEY,
    network_id BIGSERIAL NOT NULL,
    name TEXT NOT NULL,
    address character(42) NOT NULL,
    result_event_signature character(66) NOT NULL,

    CONSTRAINT fk_network
        FOREIGN KEY(network_id)
            REFERENCES Network(id),

    CONSTRAINT fk_signature
        FOREIGN KEY(result_event_signature)
            REFERENCES GameAbi(signature)
);

CREATE UNIQUE INDEX game_network_id_idx ON Game(network_id, address);
CREATE UNIQUE INDEX game_address_idx ON Game(address);
CREATE UNIQUE INDEX game_idx ON Game(network_id, name);

CREATE VIEW GameInfo AS 
    SELECT Game.id as id,
            Game.network_id as network_id,
            Game.name as name,
            Game.address as address,
            GameAbi.signature as event_signature,
            GameAbi.types as event_types,
            GameAbi.names as event_names
        FROM Game
    INNER JOIN GameAbi 
        ON Game.result_event_signature = GameAbi.signature;

CREATE TABLE IF NOT EXISTS Nickname(
    id BIGSERIAL PRIMARY KEY,
    address character(42) NOT NULL,
    nickname varchar(20) NOT NULL
);

CREATE UNIQUE INDEX nickname_idx ON Nickname(address);

CREATE TABLE IF NOT EXISTS Player(
    id BIGSERIAL PRIMARY KEY,
    address character(42) NOT NULL,
    wagered DOUBLE PRECISION NOT NULL,
    bets BIGINT NOT NULL,
    bets_won BIGINT NOT NULL,
    bets_lost BIGINT NOT NULL,
    highest_win DOUBLE PRECISION NOT NULL,
    highest_multiplier DOUBLE PRECISION NOT NULL
);

CREATE UNIQUE INDEX player_idx ON Player(address);

CREATE TABLE IF NOT EXISTS Bet(
    id BIGSERIAL PRIMARY KEY,
    transaction_hash character(66) NOT NULL UNIQUE,
    player character(42) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    game_id BIGINT NOT NULL,
    wager DECIMAL(1000, 0) NOT NULL,
    token_address character(42) NOT NULL,
    network_id BIGINT NOT NULL,
    bets BIGINT NOT NULL,
    multiplier DOUBLE PRECISION NOT NULL,
    profit DECIMAL(1000, 0) NOT NULL,

    CONSTRAINT fk_game
        FOREIGN KEY(game_id)
            REFERENCES Game(id),

    CONSTRAINT fk_network
        FOREIGN KEY(network_id)
            REFERENCES Network(id)
);

CREATE INDEX bet_player_idx ON Bet(player);
CREATE INDEX bet_game_idx ON Bet(game_id);
CREATE INDEX bet_idx ON Bet(player, game_id);
CREATE INDEX last_bets_idx ON Bet(timestamp desc);

CREATE VIEW BetInfo AS 
    SELECT Bet.id as id,
            Bet.transaction_hash as transaction_hash,
            Bet.player as player,
            Nickname.nickname as player_nickname,
            Bet.timestamp as timestamp,
            Bet.game_id as game_id,
            Game.name as game_name,
            Bet.wager as wager,
            Bet.token_address as token_address,
            Token.name as token_name,
            Bet.network_id as network_id,
            Network.name as network_name,
            Bet.bets as bets,
            Bet.multiplier as multiplier,
            Bet.profit as profit
        FROM Bet
    INNER JOIN Game
        ON Bet.game_id = Game.id
	INNER JOIN Network
        ON Bet.network_id = Network.id
	INNER JOIN Token
        ON Bet.token_address = Token.contract_address
	LEFT JOIN Nickname
        ON Bet.player = Nickname.address;


CREATE TABLE IF NOT EXISTS BanWords(
    id BIGSERIAL PRIMARY KEY,
    word TEXT
);