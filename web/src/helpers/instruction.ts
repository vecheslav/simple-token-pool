import { TOKEN_PROGRAM_ID, u64 } from '@solana/spl-token'
import { PublicKey, SYSVAR_RENT_PUBKEY, TransactionInstruction } from '@solana/web3.js'
import { config } from '../config'
import { encodeData } from '../utils/solana'
import { PoolInsructionLayouts } from './layout'

export type InitializeParams = {
  pool: PublicKey
  authority: PublicKey
  bankMint: PublicKey
  poolMint: PublicKey
  bank: PublicKey
}
export const initialize = ({ pool, authority, bankMint, poolMint, bank }: InitializeParams) => {
  const data = encodeData(PoolInsructionLayouts.Initialize)

  return new TransactionInstruction({
    keys: [
      { pubkey: pool, isSigner: true, isWritable: true },
      { pubkey: authority, isSigner: false, isWritable: false },
      { pubkey: bankMint, isSigner: false, isWritable: false },
      { pubkey: poolMint, isSigner: false, isWritable: true },
      { pubkey: bank, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    programId: new PublicKey(config.programId),
    data,
  })
}

export type SwapParams = {
  pool: PublicKey
  poolAuthority: PublicKey
  userTransferAuthority: PublicKey
  poolMint: PublicKey
  bank: PublicKey
  sender: PublicKey
  recipient: PublicKey
  amountIn: number
}
export const swap = ({
  pool,
  poolAuthority,
  userTransferAuthority,
  poolMint,
  bank,
  sender,
  recipient,
  amountIn,
}: SwapParams) => {
  const data = encodeData(PoolInsructionLayouts.Swap, { amountIn: new u64(amountIn).toBuffer() })

  return new TransactionInstruction({
    keys: [
      { pubkey: pool, isSigner: false, isWritable: false },
      { pubkey: poolAuthority, isSigner: false, isWritable: false },
      { pubkey: userTransferAuthority, isSigner: true, isWritable: false },
      { pubkey: poolMint, isSigner: false, isWritable: true },
      { pubkey: bank, isSigner: false, isWritable: true },
      { pubkey: sender, isSigner: false, isWritable: true },
      { pubkey: recipient, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    programId: new PublicKey(config.programId),
    data,
  })
}
