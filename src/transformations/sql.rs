//! `SqlTransformer`, `SqlDialect` and `Casing` (`net.datafaker.transformations.sql`).

use super::{Field, FieldName, Input, LINE_SEPARATOR, PrimitiveArray, Schema, Transformer, Value, check_finite, check_stream_limit};
use crate::{Error, Result};

/// `Casing`: how a dialect folds unquoted identifiers, which decides when an identifier needs quoting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Casing {
    /// The case of identifiers is not changed.
    Unchanged,
    /// Identifiers are converted to upper-case.
    ToUpper,
    /// Identifiers are converted to lower-case.
    ToLower,
}

impl Casing {
    /// `Casing.DEFAULT_CASING`.
    pub const DEFAULT_CASING: Casing = Casing::ToUpper;
}

/// `SqlTransformer.Case`: how keywords are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    /// `Case.CAPITAL`: `Insert Into`.
    Capital,
    /// `Case.LOWERCASE`: `insert into`.
    Lowercase,
    /// `Case.UPPERCASE`: `INSERT INTO`.
    Uppercase,
}

/// `SqlTransformer.SQLKeyWords`.
#[derive(Debug, Clone, Copy)]
enum Keyword {
    Array,
    Map,
    NamedStruct,
    InsertAll,
    InsertInto,
    Into,
    Multiset,
    Null,
    Row,
    Select1FromDual,
    Values,
}

impl Keyword {
    fn value(self, case: Case) -> &'static str {
        let (upper, lower, capital) = match self {
            Keyword::Array => ("ARRAY", "array", "Array"),
            Keyword::Map => ("MAP", "map", "Map"),
            Keyword::NamedStruct => ("NAMED_STRUCT", "named_struct", "Named_Struct"),
            Keyword::InsertAll => ("INSERT ALL", "insert all", "Insert All"),
            Keyword::InsertInto => ("INSERT INTO", "insert into", "Insert Into"),
            Keyword::Into => ("INTO", "into", "Into"),
            Keyword::Multiset => ("MULTISET", "multiset", "Multiset"),
            Keyword::Null => ("NULL", "null", "Null"),
            Keyword::Row => ("ROW", "row", "Row"),
            Keyword::Select1FromDual => ("SELECT 1 FROM dual", "select 1 from dual", "Select 1 From dual"),
            Keyword::Values => ("VALUES", "values", "Values"),
        };
        match case {
            Case::Uppercase => upper,
            Case::Lowercase => lower,
            Case::Capital => capital,
        }
    }
}

/// `SqlDialect`: the identifier quote and casing of a database, and how its batch inserts, rows, arrays and maps
/// read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    /// `SqlDialect.ANSI`.
    Ansi,
    /// `SqlDialect.BIGQUERY`.
    Bigquery,
    /// `SqlDialect.CALCITE`.
    Calcite,
    /// `SqlDialect.CLICKHOUSE`.
    Clickhouse,
    /// `SqlDialect.EXASOL`.
    Exasol,
    /// `SqlDialect.FIREBOLT`.
    Firebolt,
    /// `SqlDialect.H2`.
    H2,
    /// `SqlDialect.INFOBRIGHT`.
    Infobright,
    /// `SqlDialect.LUCIDDB`.
    Luciddb,
    /// `SqlDialect.MARIADB`.
    Mariadb,
    /// `SqlDialect.MSSQL`.
    Mssql,
    /// `SqlDialect.MYSQL`.
    Mysql,
    /// `SqlDialect.NETEZZA`.
    Netezza,
    /// `SqlDialect.ORACLE`.
    Oracle,
    /// `SqlDialect.PARACCEL`.
    Paraccel,
    /// `SqlDialect.PHOENIX`.
    Phoenix,
    /// `SqlDialect.POSTGRES`.
    Postgres,
    /// `SqlDialect.PRESTO`.
    Presto,
    /// `SqlDialect.REDSHIFT`.
    Redshift,
    /// `SqlDialect.SNOWFLAKE`.
    Snowflake,
    /// `SqlDialect.SPARKSQL`.
    Sparksql,
    /// `SqlDialect.TERADATA`.
    Teradata,
    /// `SqlDialect.VERTICA`.
    Vertica,
}

/// `DEFAULT_BEFORE_EACH_BATCH_PREFIX`.
const DEFAULT_BEFORE_EACH_BATCH_PREFIX: &str = "       ";

impl SqlDialect {
    /// Every dialect, in declaration order (`SqlDialect.values()`).
    pub const VALUES: &'static [SqlDialect] = &[
        SqlDialect::Ansi,
        SqlDialect::Bigquery,
        SqlDialect::Calcite,
        SqlDialect::Clickhouse,
        SqlDialect::Exasol,
        SqlDialect::Firebolt,
        SqlDialect::H2,
        SqlDialect::Infobright,
        SqlDialect::Luciddb,
        SqlDialect::Mariadb,
        SqlDialect::Mssql,
        SqlDialect::Mysql,
        SqlDialect::Netezza,
        SqlDialect::Oracle,
        SqlDialect::Paraccel,
        SqlDialect::Phoenix,
        SqlDialect::Postgres,
        SqlDialect::Presto,
        SqlDialect::Redshift,
        SqlDialect::Snowflake,
        SqlDialect::Sparksql,
        SqlDialect::Teradata,
        SqlDialect::Vertica,
    ];

    /// `getSqlQuoteIdentifier()`: one character, or the opening and closing one (`[]`).
    pub fn sql_quote_identifier(self) -> &'static str {
        use SqlDialect::*;
        match self {
            Ansi | Bigquery | Clickhouse | Infobright | Mariadb | Mysql | Sparksql => "`",
            Mssql => "[]",
            _ => "\"",
        }
    }

    /// `getUnquotedCasing()`.
    pub fn unquoted_casing(self) -> Casing {
        use SqlDialect::*;
        match self {
            Bigquery | Mysql | Presto | Vertica => Casing::Unchanged,
            Mariadb | Redshift => Casing::ToLower,
            _ => Casing::DEFAULT_CASING,
        }
    }

    /// `getFirstRow(dialect, columns, values, case)`: the start of a batch insert.
    fn first_row(dialect: Option<SqlDialect>, columns: &str, values: &str, case: Case) -> String {
        match dialect {
            Some(SqlDialect::Oracle) => format!(
                "{}{LINE_SEPARATOR}    {} {columns} {} {values}",
                Keyword::InsertAll.value(case),
                Keyword::Into.value(case),
                Keyword::Values.value(case)
            ),
            _ => format!("{} {columns}{LINE_SEPARATOR}{} {values}", Keyword::InsertInto.value(case), Keyword::Values.value(case)),
        }
    }

    /// `getOtherRow(dialect, columns, values, case)`: a later row of a batch insert.
    fn other_row(dialect: Option<SqlDialect>, columns: &str, values: &str, case: Case) -> String {
        match dialect {
            Some(SqlDialect::Oracle) => format!("    {} {columns} {} {values}", Keyword::Into.value(case), Keyword::Values.value(case)),
            _ => format!("{DEFAULT_BEFORE_EACH_BATCH_PREFIX}{values}"),
        }
    }

    /// `getLastRowSuffix(dialect, case)`.
    fn last_row_suffix(dialect: Option<SqlDialect>, case: Case) -> String {
        match dialect {
            Some(SqlDialect::Oracle) => format!("{LINE_SEPARATOR}{}", Keyword::Select1FromDual.value(case)),
            _ => String::new(),
        }
    }

    /// `getCompositePrefix(case)`: `ROW`, or `NAMED_STRUCT` in Spark SQL.
    pub fn composite_prefix(self, case: Case) -> &'static str {
        match self {
            SqlDialect::Sparksql => Keyword::NamedStruct.value(case),
            _ => Keyword::Row.value(case),
        }
    }

    /// `getFieldPrefix(fieldName)`: the field's name in Spark SQL (`NAMED_STRUCT('name', value, …)`), else empty.
    pub fn field_prefix(self, field_name: Option<&str>) -> Option<String> {
        match self {
            SqlDialect::Sparksql => field_name.map(str::to_string),
            _ => Some(String::new()),
        }
    }

    /// `getArrayStart()`.
    pub fn array_start(self) -> &'static str {
        if self == SqlDialect::Sparksql { "(" } else { "[" }
    }

    /// `getArrayEnd()`.
    pub fn array_end(self) -> &'static str {
        if self == SqlDialect::Sparksql { ")" } else { "]" }
    }
}

const DEFAULT_CATALOG_SEPARATOR: char = '.';

/// `SqlTransformer`: an `INSERT` statement per row, or batch inserts of several rows each.
#[derive(Debug, Clone)]
pub struct SqlTransformer {
    casing: Casing,
    quote: char,
    open_sql_identifier: char,
    close_sql_identifier: char,
    table_name: Option<String>,
    schema_name: Option<String>,
    with_batch_mode: bool,
    batch_size: i32,
    keyword_case: Case,
    force_sql_quote_identifier_usage: bool,
    dialect: Option<SqlDialect>,
}

/// `SqlTransformer.SqlTransformerBuilder`: table `MyTable`, no schema, `'` quotes, `"` identifiers, upper-case
/// casing and keywords, no batches.
#[derive(Debug, Clone)]
pub struct SqlTransformerBuilder {
    quote: char,
    sql_quote_identifier: String,
    table_name: Option<String>,
    schema_name: Option<String>,
    casing: Casing,
    with_batch_mode: bool,
    batch_size: i32,
    keyword_case: Case,
    force_sql_quote_identifier_usage: bool,
    dialect: Option<SqlDialect>,
}

impl Default for SqlTransformerBuilder {
    fn default() -> Self {
        SqlTransformerBuilder {
            quote: '\'',
            sql_quote_identifier: "\"\"".into(),
            table_name: Some("MyTable".into()),
            schema_name: Some(String::new()),
            casing: Casing::ToUpper,
            with_batch_mode: false,
            batch_size: -1,
            keyword_case: Case::Uppercase,
            force_sql_quote_identifier_usage: false,
            dialect: None,
        }
    }
}

impl SqlTransformerBuilder {
    /// `dialect(dialect)`: takes the dialect's quote and casing.
    pub fn dialect(mut self, dialect: SqlDialect) -> Self {
        self.dialect = Some(dialect);
        self
    }

    /// `casing(casing)`: also drops the dialect.
    pub fn casing(mut self, casing: Casing) -> Self {
        self.casing = casing;
        self.dialect = None;
        self
    }

    /// `quote(quote)`: the character that quotes string values.
    pub fn quote(mut self, quote: char) -> Self {
        self.quote = quote;
        self
    }

    /// `sqlQuoteIdentifier(identifier)`: also drops the dialect.
    pub fn sql_quote_identifier(mut self, sql_quote_identifier: &str) -> Self {
        self.sql_quote_identifier = sql_quote_identifier.into();
        self.dialect = None;
        self
    }

    /// `tableName(tableName)`.
    pub fn table_name(mut self, table_name: impl FieldName) -> Self {
        self.table_name = table_name.into_name();
        self
    }

    /// `schemaName(schemaName)`.
    pub fn schema_name(mut self, schema_name: impl FieldName) -> Self {
        self.schema_name = schema_name.into_name();
        self
    }

    /// `batch()`: one batch insert for all rows.
    pub fn batch(mut self) -> Self {
        self.with_batch_mode = true;
        self
    }

    /// `batch(batchSize)`: batch inserts of `batch_size` rows.
    pub fn batch_of(mut self, batch_size: i32) -> Self {
        self.batch_size = batch_size;
        self.with_batch_mode = true;
        self
    }

    /// `keywordCase(case)`.
    pub fn keyword_case(mut self, case: Case) -> Self {
        self.keyword_case = case;
        self
    }

    /// `forceUseSqlQuoteIdentifier()`: quotes every identifier, not only those that need it.
    pub fn force_use_sql_quote_identifier(mut self) -> Self {
        self.force_sql_quote_identifier_usage = true;
        self
    }

    /// `build()`: fails on an empty quote identifier, as `charAt(0)` does.
    pub fn build(self) -> Result<SqlTransformer> {
        let (identifier, casing) = match self.dialect {
            Some(d) => (d.sql_quote_identifier().to_string(), d.unquoted_casing()),
            None => (self.sql_quote_identifier.clone(), self.casing),
        };
        let chars: Vec<char> = identifier.chars().collect();
        let Some(&open) = chars.first() else {
            return Err(Error::new("StringIndexOutOfBoundsException", "Index 0 out of bounds for length 0"));
        };
        let close = if chars.len() == 1 { open } else { chars[1] };
        Ok(SqlTransformer {
            casing,
            quote: self.quote,
            open_sql_identifier: open,
            close_sql_identifier: close,
            table_name: self.table_name,
            schema_name: self.schema_name,
            with_batch_mode: self.with_batch_mode,
            batch_size: self.batch_size,
            keyword_case: self.keyword_case,
            force_sql_quote_identifier_usage: self.force_sql_quote_identifier_usage,
            dialect: self.dialect,
        })
    }
}

fn npe(message: &str) -> Error {
    Error::new("NullPointerException", message)
}

impl SqlTransformer {
    /// `SqlTransformer.builder()`.
    pub fn builder() -> SqlTransformerBuilder {
        SqlTransformerBuilder::default()
    }

    fn is_sql_quote_identifier_required_for(&self, name: &str) -> bool {
        if self.force_sql_quote_identifier_usage {
            return true;
        }
        name.chars().any(|c| {
            let unit = (c as u32 <= 0xFFFF).then_some(c as u16);
            self.casing == Casing::ToUpper && unit.is_some_and(crate::java::is_lower_case)
                || self.casing == Casing::ToLower && unit.is_some_and(crate::java::is_upper_case)
                || c == self.open_sql_identifier
                || c == self.close_sql_identifier
                || c == DEFAULT_CATALOG_SEPARATOR
        })
    }

    fn add_values<IN>(&self, input: Option<&IN>, fields: &[Field<'_, IN>], is_root: bool) -> Result<String> {
        let mut result: Vec<String> = Vec::new();
        for f in fields {
            if let Some(dialect) = self.dialect {
                let prefix = dialect
                    .field_prefix(f.name())
                    .ok_or_else(|| npe("Cannot invoke \"String.isEmpty()\" because \"fieldPrefix\" is null"))?;
                if !prefix.is_empty() && !is_root {
                    result.push(format!("{}{prefix}{}", self.quote, self.quote));
                }
            }
            if f.is_composite() {
                let dialect = self.dialect.ok_or_else(|| npe("SQL dialect is unknown, cannot use composite prefix in SQL"))?;
                result.push(format!("{}{}", dialect.composite_prefix(self.keyword_case), self.add_values(input, f.fields(), false)?));
                continue;
            }
            let value = f.transform(input)?;
            let case = self.keyword_case;
            result.push(match &value {
                Value::Null | Value::Bool(_) => value.to_java_string(),
                v if v.is_number() => v.to_java_string(),
                Value::Array(_) | Value::Primitive(_) => {
                    let dialect = self.dialect.ok_or_else(|| npe("SQL dialect is unknown, cannot use ARRAY values in SQL"))?;
                    let items = match &value {
                        Value::Primitive(p) => handle_primitives_in_array(p),
                        Value::Array(items) => self.handle_objects(items),
                        _ => unreachable!(),
                    };
                    format!("{}{}{items}{}", Keyword::Array.value(case), dialect.array_start(), dialect.array_end())
                }
                Value::Map(entries) => format!("{}({})", Keyword::Map.value(case), self.handle_map(entries)),
                Value::List(items) => format!("{}[{}]", Keyword::Multiset.value(case), self.handle_objects(items)),
                other => self.handle_object(other),
            });
        }
        let res = result.join(", ");
        Ok(if res.is_empty() { res } else { format!("({res})") })
    }

    fn handle_objects<IN>(&self, items: &[Value<'_, IN>]) -> String {
        items.iter().map(|v| self.handle_object(v)).collect::<Vec<_>>().join(", ")
    }

    fn handle_map<IN>(&self, entries: &[(Value<'_, IN>, Value<'_, IN>)]) -> String {
        entries.iter().map(|(k, v)| format!("{}, {}", self.handle_object(k), self.handle_object(v))).collect::<Vec<_>>().join(", ")
    }

    fn handle_object<IN>(&self, value: &Value<'_, IN>) -> String {
        let case = self.keyword_case;
        match value {
            Value::Null => Keyword::Null.value(case).into(),
            Value::Primitive(p) => format!("{}[{}]", Keyword::Array.value(case), handle_primitives_in_array(p)),
            Value::Array(items) => format!("{}[{}]", Keyword::Array.value(case), self.handle_objects(items)),
            Value::Map(entries) => format!("{}({})", Keyword::Map.value(case), self.handle_map(entries)),
            Value::List(items) => format!("{}[{}]", Keyword::Multiset.value(case), self.handle_objects(items)),
            other => {
                let quote_required = !other.is_number() && !matches!(other, Value::Bool(_));
                let mut res = String::new();
                for c in other.to_java_string().chars() {
                    if c == self.quote {
                        res.push('\\');
                    }
                    res.push(c);
                }
                if quote_required { format!("{}{res}{}", self.quote, self.quote) } else { res }
            }
        }
    }

    fn append_table_info<IN>(&self, fields: &[Field<'_, IN>]) -> Result<String> {
        let mut result = String::new();
        self.append_name_to_query(&mut result, self.schema_name.as_deref());
        if self.schema_name.as_deref().is_some_and(|s| !s.is_empty()) {
            result.push(DEFAULT_CATALOG_SEPARATOR);
        }
        self.append_name_to_query(&mut result, self.table_name.as_deref());
        result.push_str(" (");
        for (i, f) in fields.iter().enumerate() {
            let name = f.name().ok_or_else(|| {
                if self.force_sql_quote_identifier_usage {
                    npe("Cannot invoke \"String.length()\" because \"fieldName\" is null")
                } else {
                    npe("Cannot invoke \"String.length()\" because \"name\" is null")
                }
            })?;
            let required = self.is_sql_quote_identifier_required_for(name);
            if required {
                result.push(self.open_sql_identifier);
            }
            for c in name.chars() {
                if c == self.open_sql_identifier || c == self.close_sql_identifier {
                    result.push(self.open_sql_identifier);
                }
                result.push(c);
            }
            if required {
                result.push(self.close_sql_identifier);
            }
            if i < fields.len() - 1 {
                result.push_str(", ");
            }
        }
        result.push(')');
        Ok(result)
    }

    fn append_name_to_query(&self, sb: &mut String, name: Option<&str>) {
        let Some(name) = name.filter(|n| !n.is_empty()) else { return };
        let required = self.is_sql_quote_identifier_required_for(name);
        if required {
            sb.push(self.open_sql_identifier);
        }
        sb.push_str(name);
        if required {
            sb.push(self.close_sql_identifier);
        }
    }

    fn generate_batch_mode_statements<IN>(&self, schema: &Schema<'_, IN>, inputs: Option<&[Option<IN>]>, limit: i32) -> Result<String> {
        let mut sb = String::new();
        let limit = match inputs {
            Some(inputs) => (limit as i64).min(inputs.len() as i64) as i32,
            None => limit,
        };
        for i in 0..limit {
            let input = inputs.and_then(|inputs| inputs[i as usize].as_ref());
            sb.push_str(&self.apply_row(input, schema, i as i64)?);
            if i == limit - 1 && !sb.is_empty() || self.batch_size > 0 && (i + 1) % self.batch_size == 0 {
                sb.push_str(&SqlDialect::last_row_suffix(self.dialect, self.keyword_case));
                sb.push(';');
                if i < limit - 1 && !sb.is_empty() {
                    sb.push_str(LINE_SEPARATOR);
                }
            }
        }
        Ok(sb)
    }

    fn generate_separated_statements<IN>(&self, schema: &Schema<'_, IN>, inputs: Option<&[Option<IN>]>, limit: i32) -> Result<String> {
        let limit = match inputs {
            Some(inputs) => (limit as i64).min(inputs.len() as i64) as i32,
            None => limit,
        };
        let mut data = Vec::new();
        for i in 0..limit {
            let input = inputs.and_then(|inputs| inputs[i as usize].as_ref());
            data.push(format!("{};", self.apply(input, schema)?));
        }
        Ok(data.join(LINE_SEPARATOR))
    }

    fn unsupported() -> Error {
        Error::new(
            "UnsupportedOperationException",
            "Not supported for SQL transformer. Use generate or generateStream instead to create SQL statements.",
        )
    }
}

impl<'a, IN> Transformer<'a, IN> for SqlTransformer {
    fn apply(&self, input: Option<&IN>, schema: &Schema<'a, IN>) -> Result<String> {
        self.apply_row(input, schema, 0)
    }

    fn apply_row(&self, input: Option<&IN>, schema: &Schema<'a, IN>, row_id: i64) -> Result<String> {
        let fields = schema.fields();
        if fields.is_empty() {
            return Ok(String::new());
        }
        let case = self.keyword_case;
        if self.with_batch_mode {
            let columns = self.append_table_info(fields)?;
            let values = self.add_values(input, fields, true)?;
            if row_id == 0 || self.batch_size > 0 && row_id % self.batch_size as i64 == 0 {
                Ok(SqlDialect::first_row(self.dialect, &columns, &values, case))
            } else {
                Ok(format!(",{LINE_SEPARATOR}{}", SqlDialect::other_row(self.dialect, &columns, &values, case)))
            }
        } else {
            Ok([
                Keyword::InsertInto.value(case).to_string(),
                self.append_table_info(fields)?,
                Keyword::Values.value(case).to_string(),
                self.add_values(input, fields, true)?,
            ]
            .join(" "))
        }
    }

    fn generate_from(&self, input: &(impl Input<IN> + ?Sized), schema: &Schema<'a, IN>) -> Result<String> {
        if schema.fields().is_empty() {
            return Ok(String::new());
        }
        check_finite(input)?;
        let inputs = input.rows().collect::<Result<Vec<_>>>()?;
        let limit = inputs.len() as i32;
        if self.with_batch_mode {
            self.generate_batch_mode_statements(schema, Some(&inputs), limit)
        } else {
            self.generate_separated_statements(schema, Some(&inputs), limit)
        }
    }

    fn generate(&self, schema: &Schema<'a, IN>, limit: i32) -> Result<String> {
        if schema.fields().is_empty() {
            return Ok(String::new());
        }
        if self.with_batch_mode {
            self.generate_batch_mode_statements(schema, None, limit)
        } else {
            self.generate_separated_statements(schema, None, limit)
        }
    }

    fn start_stream(&self, _schema: &Schema<'a, IN>) -> Result<String> {
        Err(SqlTransformer::unsupported())
    }

    fn end_stream(&self) -> Result<String> {
        Err(SqlTransformer::unsupported())
    }

    /// `generateStream(schema, limit)`: a statement per item, or in batch mode a batch insert per item (an empty
    /// `;` one when `limit` is a multiple of the batch size, as upstream).
    fn generate_stream<'s>(&'s self, schema: &'s Schema<'a, IN>, limit: i64) -> Result<Box<dyn Iterator<Item = Result<String>> + 's>> {
        if schema.fields().is_empty() {
            return Ok(Box::new(std::iter::empty()));
        }
        if self.with_batch_mode {
            let batch_size = self.batch_size;
            let mut interval = (0i32, batch_size);
            return Ok(Box::new(std::iter::from_fn(move || {
                if interval.0 as i64 > limit {
                    return None;
                }
                let (start, end) = interval;
                interval = (start.wrapping_add(batch_size), end.wrapping_add(batch_size));
                let item = (|| {
                    let mut sb = String::new();
                    let mut i = start;
                    while i < end && (i as i64) < limit {
                        sb.push_str(&self.apply_row(None, schema, i as i64)?);
                        i += 1;
                    }
                    sb.push_str(&SqlDialect::last_row_suffix(self.dialect, self.keyword_case));
                    sb.push(';');
                    Ok(sb)
                })();
                Some(item)
            })));
        }
        check_stream_limit(limit)?;
        Ok(Box::new((0..limit).map(move |_| Ok(format!("{};", self.apply(None, schema)?)))))
    }
}

/// `handlePrimitivesInArray`: the elements as they print (a `char[]` prints nothing, as upstream).
fn handle_primitives_in_array(array: &PrimitiveArray) -> String {
    fn join<T>(items: &[T], f: impl Fn(&T) -> String) -> String {
        items.iter().map(f).collect::<Vec<_>>().join(", ")
    }
    match array {
        PrimitiveArray::Byte(v) => join(v, i8::to_string),
        PrimitiveArray::Short(v) => join(v, i16::to_string),
        PrimitiveArray::Boolean(v) => join(v, bool::to_string),
        PrimitiveArray::Int(v) => join(v, i32::to_string),
        PrimitiveArray::Long(v) => join(v, i64::to_string),
        PrimitiveArray::Float(v) => join(v, |f| crate::java::float_to_string(*f)),
        PrimitiveArray::Double(v) => join(v, |d| crate::java::double_to_string(*d)),
        PrimitiveArray::Char(_) => String::new(),
    }
}
