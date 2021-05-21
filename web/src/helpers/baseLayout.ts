import BufferLayout from 'buffer-layout'

export const PublicKey = (property = 'publicKey'): Record<string, unknown> => {
  return BufferLayout.blob(32, property)
}

export const Uint64 = (property = 'uint64'): Record<string, unknown> => {
  return BufferLayout.blob(8, property)
}
