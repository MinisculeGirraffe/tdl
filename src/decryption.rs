use anyhow::Error;
use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::symmetriccipher::{Decryptor, SymmetricCipherError};
use crypto::{aes, blockmodes, buffer};

static MASTER_KEY: &str = "UIlTTEMmmLfGowo/UC60x2H45W6MdGgTRfo/umg4754=";

fn decrypt(
    encrypted_data: &[u8],
    mut decryptor: Box<dyn Decryptor>,
) -> Result<Vec<u8>, SymmetricCipherError> {
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

pub fn decrypt_security_token(security_token: &str) -> Result<([u8; 16], [u8; 8]), Error> {
    let master_key = base64::decode(MASTER_KEY)?;
    let security_key = base64::decode(security_token)?;
    let iv = &security_key[..15];
    let enc_token = &security_key[16..];
    let decryptor = aes::cbc_decryptor(
        aes::KeySize::KeySize256,
        &master_key,
        iv,
        blockmodes::NoPadding,
    );
    let decrypted_token = decrypt(enc_token, decryptor).unwrap();
    let _key_slice = &decrypted_token[..15];

    let key: [u8; 16] = decrypted_token[..15].try_into()?;
    let nonce: [u8; 8] = decrypted_token[16..24].try_into()?;

    Ok((key, nonce))
}

pub async fn decrypt_file(
    file: Vec<u8>,
    key: [u8; 16],
    nonce: [u8; 8],
) -> Result<Vec<u8>, SymmetricCipherError> {
    let mut decryptor = aes::ctr(aes::KeySize::KeySize256, &key, &nonce);

    let mut final_result = Vec::new();

    let mut buffer = [0; 4096];
    let mut read_buffer = buffer::RefReadBuffer::new(&file);
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);
    loop {
        let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }
    Ok(final_result)
}
