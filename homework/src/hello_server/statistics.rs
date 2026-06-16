//! Server statisics
//! 服务器统计

use std::collections::HashMap;

/// Report for each operation
/// 每个操作的报告
#[derive(Debug)]
pub struct Report {
    _id: usize,
    key: Option<String>, // None represents invalid request
    // None 表示无效请求
}

impl Report {
    /// Creates a new report with the given id and key.
    /// 使用给定的 ID 和密钥创建一个新报告。
    pub fn new(id: usize, key: Option<String>) -> Self {
        Report { _id: id, key }
    }
}

/// Operation statisics
/// 操作统计
#[derive(Debug, Default)]
pub struct Statistics {
    hits: HashMap<Option<String>, usize>,
}

impl Statistics {
    /// Add a report to the statisics.
    /// 向统计中添加报告。
    pub fn add_report(&mut self, report: Report) {
        let hits = self.hits.entry(report.key).or_default();
        *hits += 1;
    }
}
