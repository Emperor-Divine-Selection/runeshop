#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Customer,
    Merchant,
    Operator,
}

impl Role {
    /// 写进 Valkey / metadata / proto 的唯一形态
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Customer => "customer",
            Self::Merchant => "merchant",
            Self::Operator => "operator",
        }
    }

    /// 解析失败返回 None —— 调用方决定是「登出」还是「500」
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "customer" => Some(Self::Customer),
            "merchant" => Some(Self::Merchant),
            "operator" => Some(Self::Operator),
            _ => None,
        }
    }
}
