use crate::common::{do_flush, TempDatabase};
use limbo_core::{StepResult, Value};

#[test]
fn test_last_insert_rowid_basic() -> anyhow::Result<()> {
    let _ = env_logger::try_init();
    let tmp_db = TempDatabase::new("CREATE TABLE test_rowid (id INTEGER PRIMARY KEY, val TEXT);");
    let conn = tmp_db.connect_limbo();

    // Simple insert
    let mut insert_query = conn.query("INSERT INTO test_rowid (id, val) VALUES (NULL, 'test1')")?;
    if let Some(ref mut rows) = insert_query {
        loop {
            match rows.step()? {
                StepResult::IO => {
                    tmp_db.io.run_once()?;
                }
                StepResult::Done => break,
                _ => unreachable!(),
            }
        }
    }

    // Check last_insert_rowid separately
    let mut select_query = conn.query("SELECT last_insert_rowid()")?;
    if let Some(ref mut rows) = select_query {
        loop {
            match rows.step()? {
                StepResult::Row(row) => {
                    if let Value::Integer(id) = row.values[0] {
                        assert_eq!(id, 1, "First insert should have rowid 1");
                    }
                }
                StepResult::IO => {
                    tmp_db.io.run_once()?;
                }
                StepResult::Interrupt => break,
                StepResult::Done => break,
                StepResult::Busy => panic!("Database is busy"),
            }
        }
    }

    // Test explicit rowid
    match conn.query("INSERT INTO test_rowid (id, val) VALUES (5, 'test2')") {
        Ok(Some(ref mut rows)) => loop {
            match rows.step()? {
                StepResult::IO => {
                    tmp_db.io.run_once()?;
                }
                StepResult::Done => break,
                _ => unreachable!(),
            }
        },
        Ok(None) => {}
        Err(err) => eprintln!("{}", err),
    };

    // Check last_insert_rowid after explicit id
    let mut last_id = 0;
    match conn.query("SELECT last_insert_rowid()") {
        Ok(Some(ref mut rows)) => loop {
            match rows.step()? {
                StepResult::Row(row) => {
                    if let Value::Integer(id) = row.values[0] {
                        last_id = id;
                    }
                }
                StepResult::IO => {
                    tmp_db.io.run_once()?;
                }
                StepResult::Interrupt => break,
                StepResult::Done => break,
                StepResult::Busy => panic!("Database is busy"),
            }
        },
        Ok(None) => {}
        Err(err) => eprintln!("{}", err),
    };
    assert_eq!(last_id, 5, "Explicit insert should have rowid 5");
    do_flush(&conn, &tmp_db)?;
    Ok(())
}
