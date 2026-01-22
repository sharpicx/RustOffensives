#[derive(Debug)]
pub enum TcpState {
    Closed = 1,
    Listen = 2,
    SynSent = 3,
    SynReceived = 4,
    Established = 5,
    FinWait1 = 6,
    FinWait2 = 7,
    CloseWait = 8,
    Closing = 9,
    LastAck = 10,
    TimeWait = 11,
    DeleteTcb = 12,
}

impl From<u32> for TcpState {
    fn from(state: u32) -> Self {
        match state {
            1 => TcpState::Closed,
            2 => TcpState::Listen,
            3 => TcpState::SynSent,
            4 => TcpState::SynReceived,
            5 => TcpState::Established,
            6 => TcpState::FinWait1,
            7 => TcpState::FinWait2,
            8 => TcpState::CloseWait,
            9 => TcpState::Closing,
            10 => TcpState::LastAck,
            11 => TcpState::TimeWait,
            12 => TcpState::DeleteTcb,
            _ => TcpState::Closed,
        }
    }
}

impl std::fmt::Display for TcpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TcpState::Closed => "CLOSED",
            TcpState::Listen => "LISTEN",
            TcpState::SynSent => "SYNSENT",
            TcpState::SynReceived => "SYNRECEIVED",
            TcpState::Established => "ESTABLISHED",
            TcpState::FinWait1 => "FINWAIT1",
            TcpState::FinWait2 => "FINWAIT2",
            TcpState::CloseWait => "CLOSEWAIT",
            TcpState::Closing => "CLOSING",
            TcpState::LastAck => "LASTACK",
            TcpState::TimeWait => "TIMEWAIT",
            TcpState::DeleteTcb => "DELETETCB",
        };
        write!(f, "{}", s)
    }
}
