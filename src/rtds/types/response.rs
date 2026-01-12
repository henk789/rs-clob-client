use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{Address, Decimal};

/// Top-level RTDS message wrapper.
///
/// All messages received from the RTDS WebSocket connection are deserialized into this struct.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct RtdsMessage {
    /// The subscription topic (e.g., `crypto_prices`, `comments`)
    pub topic: String,
    /// The message type/event (e.g., `update`, `comment_created`)
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
    /// Event-specific data object
    pub payload: Value,
}

impl RtdsMessage {
    /// Try to extract the payload as a crypto price update.
    #[must_use]
    pub fn as_crypto_price(&self) -> Option<CryptoPrice> {
        if self.topic == "crypto_prices" {
            serde_json::from_value(self.payload.clone()).ok()
        } else {
            None
        }
    }

    /// Try to extract the payload as a Chainlink price update.
    #[must_use]
    pub fn as_chainlink_price(&self) -> Option<ChainlinkPrice> {
        if self.topic == "crypto_prices_chainlink" {
            serde_json::from_value(self.payload.clone()).ok()
        } else {
            None
        }
    }

    /// Try to extract the payload as a trade event.
    #[must_use]
    pub fn as_trade(&self) -> Option<Trade> {
        if self.topic == "activity" && self.msg_type == "trades" {
            serde_json::from_value(self.payload.clone()).ok()
        } else {
            None
        }
    }

    /// Try to extract the payload as a comment event.
    #[must_use]
    pub fn as_comment(&self) -> Option<Comment> {
        if self.topic == "comments" {
            serde_json::from_value(self.payload.clone()).ok()
        } else {
            None
        }
    }
}

/// Binance crypto price update payload.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize, Builder)]
pub struct CryptoPrice {
    /// Trading pair symbol (lowercase concatenated, e.g., "solusdt", "btcusdt")
    pub symbol: String,
    /// Price timestamp in Unix milliseconds
    pub timestamp: i64,
    /// Current price value
    pub value: Decimal,
}

/// Chainlink price feed update payload.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize, Builder)]
pub struct ChainlinkPrice {
    /// Trading pair symbol (slash-separated, e.g., "eth/usd", "btc/usd")
    pub symbol: String,
    /// Price timestamp in Unix milliseconds
    pub timestamp: i64,
    /// Current price value
    pub value: Decimal,
}

/// Trade event payload.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Trade {
    /// Asset identifier
    pub asset: String,
    /// Bio of the trader
    pub bio: String,
    /// Condition ID of the trade
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    /// Event slug
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    /// Icon URL
    pub icon: String,
    /// Trader's name
    pub name: String,
    /// Outcome of the trade
    pub outcome: String,
    /// Outcome index
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i64,
    /// Price of the trade
    pub price: Decimal,
    /// Profile image URL
    #[serde(rename = "profileImage")]
    pub profile_image: String,
    /// Proxy wallet address
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    /// Pseudonym of the trader
    pub pseudonym: String,
    /// Side of the trade (e.g., "BUY", "SELL")
    pub side: String,
    /// Size of the trade
    pub size: Decimal,
    /// Slug of the trade
    pub slug: String,
    /// Timestamp of the trade in Unix seconds
    pub timestamp: i64,
    /// Title of the event
    pub title: String,
    /// Transaction hash
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
}

/// Comment event payload.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize, Builder)]
pub struct Comment {
    /// Unique identifier for this comment
    pub id: String,
    /// The text content of the comment
    pub body: String,
    /// ISO 8601 timestamp when the comment was created
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    /// ID of the parent comment if this is a reply (null for top-level comments)
    #[serde(rename = "parentCommentID", default)]
    pub parent_comment_id: Option<String>,
    /// ID of the parent entity (event, market, etc.)
    #[serde(rename = "parentEntityID")]
    pub parent_entity_id: i64,
    /// Type of parent entity (e.g., "Event", "Market")
    #[serde(rename = "parentEntityType")]
    pub parent_entity_type: String,
    /// Profile information of the user who created the comment
    pub profile: CommentProfile,
    /// Current number of reactions on this comment
    #[serde(rename = "reactionCount", default)]
    pub reaction_count: i64,
    /// Polygon address for replies
    #[serde(rename = "replyAddress", default)]
    pub reply_address: Option<Address>,
    /// Current number of reports on this comment
    #[serde(rename = "reportCount", default)]
    pub report_count: i64,
    /// Polygon address of the user who created the comment
    #[serde(rename = "userAddress")]
    pub user_address: Address,
}

/// Profile information for a comment author.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize, Builder)]
pub struct CommentProfile {
    /// User profile address
    #[serde(rename = "baseAddress")]
    pub base_address: Address,
    /// Whether the username should be displayed publicly
    #[serde(rename = "displayUsernamePublic", default)]
    pub display_username_public: bool,
    /// User's display name
    pub name: String,
    /// Proxy wallet address used for transactions
    #[serde(rename = "proxyWallet", default)]
    pub proxy_wallet: Option<Address>,
    /// Generated pseudonym for the user
    #[serde(default)]
    pub pseudonym: Option<String>,
}

/// Comment message types.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentType {
    /// New comment created
    CommentCreated,
    /// Comment was removed/deleted
    CommentRemoved,
    /// Reaction added to a comment
    ReactionCreated,
    /// Reaction removed from a comment
    ReactionRemoved,
    /// Unknown comment type from the API (captures the raw value for debugging).
    #[serde(untagged)]
    Unknown(String),
}

/// Deserialize messages from the byte slice.
///
/// Handles both single objects and arrays of messages.
/// Returns an empty vector for empty or whitespace-only input.
pub fn parse_messages(bytes: &[u8]) -> crate::Result<Vec<RtdsMessage>> {
    // Handle empty or whitespace-only input (server keepalive messages)
    let trimmed = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .map_or(&[][..], |start| &bytes[start..]);

    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // Try parsing as array first, fall back to single object
    if trimmed.first() == Some(&b'[') {
        Ok(serde_json::from_slice(trimmed)?)
    } else {
        let msg: RtdsMessage = serde_json::from_slice(trimmed)?;
        Ok(vec![msg])
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn parse_crypto_price_message() {
        let json = r#"{
            "topic": "crypto_prices",
            "type": "update",
            "timestamp": 1753314064237,
            "payload": {
                "symbol": "solusdt",
                "timestamp": 1753314064213,
                "value": 189.55
            }
        }"#;

        let msgs = parse_messages(json.as_bytes()).unwrap();
        assert_eq!(msgs.len(), 1);

        let msg = &msgs[0];
        assert_eq!(msg.topic, "crypto_prices");
        assert_eq!(msg.msg_type, "update");

        let price = msg.as_crypto_price().unwrap();
        assert_eq!(price.symbol, "solusdt");
        assert_eq!(price.value, dec!(189.55));
    }

    #[test]
    fn parse_chainlink_price_message() {
        let json = r#"{
            "topic": "crypto_prices_chainlink",
            "type": "update",
            "timestamp": 1753314064237,
            "payload": {
                "symbol": "eth/usd",
                "timestamp": 1753314064213,
                "value": 3456.78
            }
        }"#;

        let msgs = parse_messages(json.as_bytes()).unwrap();
        assert_eq!(msgs.len(), 1);

        let msg = &msgs[0];
        assert_eq!(msg.topic, "crypto_prices_chainlink");

        let price = msg.as_chainlink_price().unwrap();
        assert_eq!(price.symbol, "eth/usd");
        assert_eq!(price.value, dec!(3456.78));
    }

    #[test]
    fn parse_trade_message() {
        let json = r#"{
            "topic": "activity",
            "type": "trades",
            "timestamp": 1753314064237,
            "payload": {
                "asset": "0xabc123",
                "bio": "Trader bio",
                "conditionId": "cond-001",
                "eventSlug": "event-xyz",
                "icon": "https://example.com/icon.png",
                "name": "TraderName",
                "outcome": "Up",
                "outcomeIndex": 1,
                "price": 0.25,
                "profileImage": "https://example.com/profile.png",
                "proxyWallet": "0xdef456",
                "pseudonym": "TraderPseudo",
                "side": "BUY",
                "size": 10.0,
                "slug": "trade-slug",
                "timestamp": 1753314064,
                "title": "Trade Title",
                "transactionHash": "0xhashvalue"
            }
        }"#;
        let msgs = parse_messages(json.as_bytes()).unwrap();
        assert_eq!(msgs.len(), 1);

        let msg = &msgs[0];
        assert_eq!(msg.topic, "activity");
        assert_eq!(msg.msg_type, "trades");

        let trade = msg.as_trade().unwrap();
        assert_eq!(trade.asset, "0xabc123");
        assert_eq!(trade.price, dec!(0.25));
        assert_eq!(trade.size, dec!(10.0));
    }

    #[test]
    fn parse_comment_message() {
        let json = r#"{
            "topic": "comments",
            "type": "comment_created",
            "timestamp": 1753454975808,
            "payload": {
                "body": "Test comment",
                "createdAt": "2025-07-25T14:49:35.801298Z",
                "id": "1763355",
                "parentCommentID": "1763325",
                "parentEntityID": 18396,
                "parentEntityType": "Event",
                "profile": {
                    "baseAddress": "0xce533188d53a16ed580fd5121dedf166d3482677",
                    "displayUsernamePublic": true,
                    "name": "salted.caramel",
                    "proxyWallet": "0x4ca749dcfa93c87e5ee23e2d21ff4422c7a4c1ee",
                    "pseudonym": "Adored-Disparity"
                },
                "reactionCount": 0,
                "replyAddress": "0x0bda5d16f76cd1d3485bcc7a44bc6fa7db004cdd",
                "reportCount": 0,
                "userAddress": "0xce533188d53a16ed580fd5121dedf166d3482677"
            }
        }"#;

        let msgs = parse_messages(json.as_bytes()).unwrap();
        assert_eq!(msgs.len(), 1);

        let msg = &msgs[0];
        assert_eq!(msg.topic, "comments");
        assert_eq!(msg.msg_type, "comment_created");

        let comment = msg.as_comment().unwrap();
        assert_eq!(comment.id, "1763355");
        assert_eq!(comment.body, "Test comment");
        assert_eq!(comment.profile.name, "salted.caramel");
    }

    #[test]
    fn parse_message_array() {
        let json = r#"[{
            "topic": "crypto_prices",
            "type": "update",
            "timestamp": 1753314064237,
            "payload": {
                "symbol": "btcusdt",
                "timestamp": 1753314064213,
                "value": 67234.50
            }
        }]"#;

        let msgs = parse_messages(json.as_bytes()).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].topic, "crypto_prices");
    }

    #[test]
    fn parse_empty_input() {
        let msgs = parse_messages(b"").unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn parse_whitespace_only_input() {
        let msgs = parse_messages(b"   \n\t  ").unwrap();
        assert!(msgs.is_empty());
    }
}
