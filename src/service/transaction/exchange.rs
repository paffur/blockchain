extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::{PublicKey, Signature, verify};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::asset::Asset;
use service::transaction::TRANSACTION_FEE;

use super::{SERVICE_ID, TX_EXCHANGE_ID};
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

encoding_struct! {
    struct ExchangeOffer {
        const SIZE = 97;

        field sender:                 &PublicKey   [00 => 32]
        field sender_assets:          Vec<Asset>   [32 => 40]
        field sender_value:           u64          [40 => 48]

        field recipient:              &PublicKey   [48 => 80]
        field recipient_assets:       Vec<Asset>   [80 => 88]
        field recipient_value:        u64          [88 => 96]

        field fee_strategy:           u8           [96 => 97]
    }
}

message! {
    struct TxExchange {
        const TYPE = SERVICE_ID;
        const ID = TX_EXCHANGE_ID;
        const SIZE = 80;

        field offer:             ExchangeOffer     [00 => 8]
        field seed:              u64               [8 => 16]
        field sender_signature:  &Signature        [16 => 80]
    }
}

impl TxExchange {
    pub fn get_offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    pub fn get_fee(&self) -> u64 {
        TRANSACTION_FEE
    }
}

impl Transaction for TxExchange {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        *self.offer().sender() != *self.offer().recipient() &&
            self.verify_signature(self.offer().recipient()) &&
            verify(
                self.sender_signature(),
                &self.offer().raw,
                self.offer().sender(),
            )

    }

    fn execute(&self, view: &mut Fork) {
        let mut tx_status = TxStatus::Fail;
        WalletSchema::map(view, |mut schema| {
            let sender = schema.wallet(self.offer().sender());
            let recipient = schema.wallet(self.offer().recipient());
            if let (Some(mut sender), Some(mut recipient)) = (sender, recipient) {
                if sender.balance() >= self.offer().sender_value() &&
                    sender.in_wallet_assets(&self.offer().sender_assets()) &&
                    recipient.balance() >= self.offer().recipient_value() &&
                    recipient.in_wallet_assets(&self.offer().recipient_assets())
                {
                    println!("--   Exchange transaction   --");
                    println!("Sender's balance before transaction : {:?}", sender);
                    println!("Recipient's balance before transaction : {:?}", recipient);

                    sender.decrease(self.offer().sender_value());
                    recipient.increase(self.offer().sender_value());

                    sender.increase(self.offer().recipient_value());
                    recipient.decrease(self.offer().recipient_value());

                    sender.del_assets(&self.offer().sender_assets());
                    recipient.add_assets(self.offer().sender_assets());

                    sender.add_assets(self.offer().recipient_assets());
                    recipient.del_assets(&self.offer().recipient_assets());

                    println!("Sender's balance before transaction : {:?}", sender);
                    println!("Recipient's balance before transaction : {:?}", recipient);
                    let mut wallets = schema.wallets();
                    wallets.put(self.offer().sender(), sender);
                    wallets.put(self.offer().recipient(), recipient);
                    tx_status = TxStatus::Success;
                }
            }
        });

        TxStatusSchema::map(view, |mut db| db.set_status(&self.hash(), tx_status))
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "tx_fee": 0,

        })
    }
}

#[cfg(test)]
mod tests {
    use super::TxExchange;
    use exonum::blockchain::Transaction;

    fn get_json() -> String {
        r#"{
            "body": {
                "offer": {
                "sender": "d350490ebf5d5afe3ddb36fcde58c1b4874792c46c85d3f3d7a3f3509c2acb60",
                "sender_assets": [
                    {
                    "hash_id": "67e5504410b1426f9247bb680e5fe0c8",
                    "amount": 5
                    },
                    {
                    "hash_id": "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8",
                    "amount": 7
                    }
                ],
                "sender_value": "37",
                "recipient": "b9426d175f946ed39211e5ca4dad1856d83caf92211661d94c660ba85c6f90be",
                "recipient_assets": [
                    {
                    "hash_id": "8d7d6d5d4d3d2d1d2c1c2b1b4a3a2a1a",
                    "amount": 1
                    }
                ],
                "recipient_value": "0",
                "fee_strategy": 1
                },
                "seed": "106",
                "sender_signature": "00c8ff68efd309ba5a65c44d341e8cb130cf4be6b6eb67b12bc6d373c7776be2260105f35f408d02553269ed0c46c6a94ad44d5f078b780e98fadd12e78db20c"
            },
            "network_id": 0,
            "protocol_version": 0,
            "service_id": 2,
            "message_id": 6,
            "signature": "87d225e432a99b1efc9d32e9133577f211db5a2610c4929ff9348cc56e3ee5cde4a10311a197b0db49d987c5529c76c8e3740078f4625f77530f86575418450c"
        }"#.to_string()
    }

    #[test]
    fn test_exchange_info() {
        let tx: TxExchange = ::serde_json::from_str(&get_json()).unwrap();
        assert_eq!(0, tx.info()["tx_fee"]);
    }
}
