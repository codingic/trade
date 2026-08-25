use anyhow::Result;
use rusqlite::Connection;

pub(crate) fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS klines (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol          TEXT    NOT NULL,
            interval        TEXT    NOT NULL,
            open_time       INTEGER NOT NULL,
            close_time      INTEGER NOT NULL,
            open            REAL    NOT NULL,
            high            REAL    NOT NULL,
            low             REAL    NOT NULL,
            close           REAL    NOT NULL,
            volume          REAL    NOT NULL,
            quote_volume    REAL    NOT NULL,
            trades          INTEGER NOT NULL,
            taker_buy_base  REAL    NOT NULL,
            taker_buy_quote REAL    NOT NULL,
            UNIQUE(symbol, interval, open_time)
        );",
    )?;
    Ok(())
}