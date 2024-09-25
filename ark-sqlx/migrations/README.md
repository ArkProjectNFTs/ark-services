## Caution

When creating a new table, ensure that the `arkproject` user has the necessary privileges by adding the following SQL command at the end of your SQL script:

```sql
GRANT ALL PRIVILEGES ON TABLE your_table TO "arkproject";
