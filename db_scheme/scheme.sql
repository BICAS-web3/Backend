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
DROP TABLE IF EXISTS PancakeAddress CASCADE;
DROP VIEW IF EXISTS BetInfo;
DROP VIEW IF EXISTS VIEW;

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

CREATE TABLE IF NOT EXISTS TokenPrice(
    id BIGSERIAL PRIMARY KEY,
    token_name TEXT NOT NULL,
    price DOUBLE PRECISION
);

CREATE UNIQUE INDEX token_price_idx ON TokenPrice(token_name);

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

--CREATE UNIQUE INDEX game_network_id_idx ON Game(network_id, address);
--CREATE UNIQUE INDEX game_address_idx ON Game(address);
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

CREATE TABLE IF NOT EXISTS Referals(
    id BIGSERIAL PRIMARY KEY,
    refer_to character(42) NOT NULL,
    referal character(42) NOT NULL
);

CREATE UNIQUE INDEX referal_idx ON Referals(referal);

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


CREATE TABLE IF NOT EXISTS PancakeAddress(
    id BIGSERIAL PRIMARY KEY,
    address TEXT NOT NULL,
    usdt_address TEXT NOT NULL,
    network_id BIGINT NOT NULL,


    CONSTRAINT fk_network
        FOREIGN KEY(network_id)
            REFERENCES Network(id)
);

CREATE UNIQUE INDEX pancake_idx ON PancakeAddress(network_id);

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

CREATE VIEW Totals AS
    SELECT 
        COUNT(bet.id) AS bets_amount,
        COUNT(DISTINCT bet.player) AS player_amount,
        (SELECT 
            SUM((bet.wager/1000000000000000000)*price.price)
                from bet
                INNER JOIN (SELECT 
                    token.name AS name,
                    token.contract_address AS address,
                    tokenprice.price AS price
    FROM token
    INNER JOIN tokenprice ON token.name=tokenprice.token_name) AS price
            ON bet.token_address = price.address)
    FROM bet;

CREATE TABLE IF NOT EXISTS BanWords(
    id BIGSERIAL PRIMARY KEY,
    word TEXT
);


-- INITIAL DATA

-- nativecurrency
INSERT INTO public.nativecurrency(
	name, symbol, decimals)
	VALUES ('ETH', 'ETH', 18);
INSERT INTO public.nativecurrency(
	name, symbol, decimals)
	VALUES ('BNB', 'BNB', 18);
INSERT INTO public.nativecurrency(
	name, symbol, decimals)
	VALUES ('ETH', 'ETH', 18);

-- network
-- INSERT INTO public.network(
-- 	id, name, short_name, native_currency_id)
-- 	VALUES (97, 'BSC TestNet', 'BSCT', 2);
-- INSERT INTO public.network(
-- 	id, name, short_name, native_currency_id)
-- 	VALUES (5, 'Goerli', 'Goerli', 3);
INSERT INTO public.network(
	id, name, short_name, native_currency_id)
	VALUES (56, 'Binance smart chain', 'BSC', 2);
INSERT INTO public.network(
	id, name, short_name, native_currency_id)
	VALUES (42161, 'Arbitrum', 'ARB', 1);

-- rpc
-- INSERT INTO public.rpcurl(
-- 	network_id, url)
-- 	VALUES (97, 'https://data-seed-prebsc-1-s1.binance.org:8545');
-- INSERT INTO public.rpcurl(
-- 	network_id, url)
-- 	VALUES (5, 'https://goerli.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161');
INSERT INTO public.rpcurl(
	network_id, url)
	VALUES (56, 'https://bsc-mainnet.nodereal.io/v1/64a9df0874fb4a93b9d0a3849de012d3');
INSERT INTO public.rpcurl(
	network_id, url)
	VALUES (42161, 'https://arb1.arbitrum.io/rpc');

-- blockexplorer
-- INSERT INTO public.blockexplorerurl(
-- 	network_id, url)
-- 	VALUES (97, 'https://testnet.bscscan.com');
-- INSERT INTO public.blockexplorerurl(
-- 	network_id, url)
-- 	VALUES (5, 'https://goerli.etherscan.io');
INSERT INTO public.blockexplorerurl(
	network_id, url)
	VALUES (56, 'https://bscscan.com');
INSERT INTO public.blockexplorerurl(
	network_id, url)
	VALUES (42161, 'https://arbiscan.io');

-- token
-- INSERT INTO public.token(
-- 	network_id, name, contract_address)
-- 	VALUES (97, 'DRAX', '0xbdc4e7171743f6f1b52b6a7d479ed32d8ffcf018');
-- INSERT INTO public.token(
-- 	network_id, name, contract_address)
-- 	VALUES (5, 'DRAX', '0xa9a9a2f699537806197baa5e0ba2f0ec4336c825');
INSERT INTO public.token(
	network_id, name, contract_address)
	VALUES (56, 'DRAX', '0x7f7f49b6128f7cb89baab704f6ea1662a270455b');
INSERT INTO public.token(
	network_id, name, contract_address)
	VALUES (42161, 'ARB', '0x912ce59144191c1204e64559fe8253a0e49e6548');

-- pancakeaddress
INSERT INTO public.pancakeaddress(
	address, usdt_address, network_id)
	VALUES ('0x10ED43C718714eb63d5aA57B78B54704E256024E', '0x55d398326f99059fF775485246999027B3197955', '97');

-- gameabi
INSERT INTO public.gameabi(
	signature, types, names)
	VALUES (
        '0x063ba2c91a70f945b84c24531b0de813d66f430987169cb7d878431c04cb0004', 
        '["uint256", "uint256", "address", "uint8[]", "uint256[]", "uint32"]', 
        'wager payout tokenAddress coinOutcomes payouts numGames'
    );
INSERT INTO public.gameabi(
	signature, types, names)
	VALUES (
        '0x090dbd65630d04a5178ecb9346e0cdcc299215a135c6eb7ecd530ce00dfa44d2', 
        '["uint256","uint256","address","uint256[]","uint256[]","uint32"]', 
        'wager payout tokenAddress diceOutcomes payouts numGames'
    );
INSERT INTO public.gameabi(
	signature, types, names)
	VALUES (
        '0x10926c19b020b305e529b4fbe64764ce71360378f742c3e3d04e62d586bf9c0e', 
        '["uint256","uint256","address","uint8[]","uint8[]","uint256[]","uint32"]', 
        'wager payout tokenAddress outcomes randomActions payouts numGames'
    );
INSERT INTO public.gameabi(
	signature, types, names)
	VALUES (
        '0xb73b6b634aea9965e3c60d60ac8d2380100c337b6d66167d297351746f4f1ac9', 
        '["uint256","uint256","address","uint8[10]","uint256"]', 
        'wager payout tokenAddress playerHand outcome'
    );
INSERT INTO public.gameabi(
	signature, types, names)
	VALUES (
        '0xc3b36130c75d38724a3591fd74cfe9738bf5234994a2bebe0d81ec71e012282a', 
        '["uint8[10]"]', 
        'playerHand'
    );

-- game
-- INSERT INTO public.game(
-- 	network_id, name, address, result_event_signature)
-- 	VALUES (
--         97,
--         'CoinFlip',
--         '0xa81bbcb6807fb63c0e7dbb1289ef5fe02410b512', 
--         '0x063ba2c91a70f945b84c24531b0de813d66f430987169cb7d878431c04cb0004'
--     );
-- INSERT INTO public.game(
-- 	network_id, name, address, result_event_signature)
-- 	VALUES (
--         5,
--         'CoinFlip',
--         '0x1c5e8342A3c8a11c4726C7eA5E63CDF93c00B93C', 
--         '0x063ba2c91a70f945b84c24531b0de813d66f430987169cb7d878431c04cb0004'
--     );
-- INSERT INTO public.game(
-- 	network_id, name, address, result_event_signature)
-- 	VALUES (
--         5,
--         'Dice',
--         '0x5070ac920B5c19aC5b87EE0E355D9c690be0bd46', 
--         '0x090dbd65630d04a5178ecb9346e0cdcc299215a135c6eb7ecd530ce00dfa44d2'
--     );
-- INSERT INTO public.game(
-- 	network_id, name, address, result_event_signature)
-- 	VALUES (
--         5,
--         'RockPaperScissors',
--         '0x02C3284378488eF235fE04D5E2E5Af4e36b5dCf4', 
--         '0x10926c19b020b305e529b4fbe64764ce71360378f742c3e3d04e62d586bf9c0e'
--     );
-- INSERT INTO public.game(
-- 	network_id, name, address, result_event_signature)
-- 	VALUES (
--         5,
--         'Poker',
--         '0xD91e9c8b0B77bDd8cd044B84ED70a8bC21bCaE87', 
--         '0xb73b6b634aea9965e3c60d60ac8d2380100c337b6d66167d297351746f4f1ac9'
--     );
-- INSERT INTO public.game(
-- 	network_id, name, address, result_event_signature)
-- 	VALUES (
--         5,
--         'PokerStart',
--         '0xD91e9c8b0B77bDd8cd044B84ED70a8bC21bCaE87', 
--         '0xc3b36130c75d38724a3591fd74cfe9738bf5234994a2bebe0d81ec71e012282a'
--     );

INSERT INTO public.game(
	network_id, name, address, result_event_signature)
	VALUES (
        56,
        'Poker',
        '0x64B72C684364Df7aAdb8716ddE51650A9131436f', 
        '0xb73b6b634aea9965e3c60d60ac8d2380100c337b6d66167d297351746f4f1ac9'
    );
INSERT INTO public.game(
	network_id, name, address, result_event_signature)
	VALUES (
        56,
        'PokerStart',
        '0x64B72C684364Df7aAdb8716ddE51650A9131436f', 
        '0xc3b36130c75d38724a3591fd74cfe9738bf5234994a2bebe0d81ec71e012282a'
    );

INSERT INTO public.game(
	network_id, name, address, result_event_signature)
	VALUES (
        42161,
        'Poker',
        '0xF020cf34ab78086524199BD92c9F8eDe55126480', 
        '0xb73b6b634aea9965e3c60d60ac8d2380100c337b6d66167d297351746f4f1ac9'
    );
INSERT INTO public.game(
	network_id, name, address, result_event_signature)
	VALUES (
        42161,
        'PokerStart',
        '0xF020cf34ab78086524199BD92c9F8eDe55126480', 
        '0xc3b36130c75d38724a3591fd74cfe9738bf5234994a2bebe0d81ec71e012282a'
    );
