use anchor_lang::prelude::*;

const CRANK_SIZE: u16 = 4096;

// Ring Buffer Crank. Fifo, first-in-first-out
#[account(zero_copy)]
pub struct RingBufferCrank {
    pub instrmt_grp: Pubkey, // Instrument group rb-crank belongs to.
    pub cranks: [Crank; CRANK_SIZE as usize],
    pub head: u16, // First element inserted.
    pub next: u16, // Next avail element.
    pub padding: [u8; 4],
}

#[cfg(test)]
impl RingBufferCrank {
    pub fn new() -> Self {
        Self {
            instrmt_grp: Pubkey::default(),
            cranks: [Crank::new_empty(); CRANK_SIZE as usize],
            next: 1,
            head: 0,
            padding: [0u8; 4],
        }
    }
}
#[cfg(test)]
impl Crank {
    pub fn new_empty() -> Self {
        Self {
            maker: Pubkey::default(),
            vault: Pubkey::default(),
            token_acc: Pubkey::default(),
            quantity: 0,
        }
    }
}

impl RingBufferCrank {
    pub fn insert(
        &mut self,
        vault: Pubkey,
        token_account: Pubkey,
        is_buy: bool,
        maker: Pubkey,
        quantity: u64,
        limit: u64,
    ) {
        if self.head == self.next {
            panic!("rb-crank filled up. Crank faster to accept new orders!");
        }

        let crank = match is_buy {
            true => Crank::new(
                maker,
                vault,
                token_account,
                quantity.checked_add(limit).unwrap(),
            ),
            false => Crank::new(maker, vault, token_account, quantity),
        };

        self.cranks[self.next as usize] = crank;
        self.next = (self.next + 1) % CRANK_SIZE
    }

    pub fn remove_head(&mut self) -> Crank {
        let head = self.cranks[self.head as usize].clone();
        self.cranks[self.head as usize].clear();
        self.head = (self.head + 1) & CRANK_SIZE;
        head
    }

    pub fn space() -> usize {
        CRANK_SIZE as usize * Crank::space() + 1 + 7
    }
}

#[zero_copy]
pub struct Crank {
    maker: Pubkey,     // Maker.
    vault: Pubkey,     // Mint to transfer to maker.
    token_acc: Pubkey, // Mint to transfer to maker.
    quantity: u64,     // Total quantity filled.
}

impl Crank {
    pub fn new(maker: Pubkey, vault: Pubkey, token_acc: Pubkey, quantity: u64) -> Self {
        Self {
            maker,
            vault,
            quantity,
            token_acc,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.quantity == 0
    }

    pub fn get_maker(&self) -> Pubkey {
        self.maker
    }

    pub fn get_token_account(&self) -> Pubkey {
        self.token_acc
    }

    pub fn get_vault(&self) -> Pubkey {
        self.vault
    }

    pub fn get_quantity(&self) -> u64 {
        self.quantity
    }

    pub fn clear(&mut self) {
        self.maker = Pubkey::default();
        self.vault = Pubkey::default();
        self.token_acc = Pubkey::default();
        self.quantity = 0;
    }

    pub fn space() -> usize {
        32 * 2 + 8
    }
}
