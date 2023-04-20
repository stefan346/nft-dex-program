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
            mint: Pubkey::default(),
            quantity: 0,
        }
    }
}

impl RingBufferCrank {
    pub fn insert(
        &mut self,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        is_buy: bool,
        maker: Pubkey,
        quantity: u64,
        price: u64,
    ) {
        if self.head == self.next {
            panic!("rb-crank filled up. Crank faster to accept new orders!");
        }
        
        let crank = match is_buy {
            true => {
                Crank::new(maker, quote_mint, quantity.checked_add(price).unwrap())
            },
            false => {
                Crank::new(maker, base_mint, quantity)
            }
        };

        self.cranks[self.next as usize] = crank;
        self.next = (self.next + 1) % CRANK_SIZE
    }

    pub fn remove_head(&mut self) {
        self.cranks[self.head as usize].clear();
        self.head = (self.head + 1) & CRANK_SIZE;
    }

    pub fn space() -> usize {
        CRANK_SIZE as usize * Crank::space() + 1 + 7
    }
}

#[zero_copy]
pub struct Crank {
    pub maker: Pubkey, // Maker.
    pub mint: Pubkey,  // Mint to transfer to maker.
    pub quantity: u64, // Total quantity filled.
}

impl Crank {
    pub fn new(maker: Pubkey, mint: Pubkey, quantity: u64) -> Self {
        Self {
            maker,
            mint,
            quantity,
        }
    }

    pub fn clear(&mut self) {
        self.maker = Pubkey::default();
        self.mint = Pubkey::default();
        self.quantity = 0;
    }

    pub fn space() -> usize {
        32 * 2 + 8
    }
}
