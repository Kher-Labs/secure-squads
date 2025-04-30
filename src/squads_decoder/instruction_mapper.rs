use super::ParseableInstruction;
use super::discriminator::discriminator_from_ix;
use chainparser::ChainparserDeserialize;
use chainparser::errors::{ChainparserError, ChainparserResult};
use lazy_static::lazy_static;
use log::trace;
use serde_json::{Map, Value};
use solana_idl::Idl;
use solana_idl::{EnumFields, IdlInstruction, IdlType, IdlTypeDefinitionTy};
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};
#[rustfmt::skip]
lazy_static! {
    pub static ref BUILTIN_PROGRAMS: HashMap<Pubkey, &'static str> = [
        ("System Program"                , "11111111111111111111111111111111")           ,
        ("BPF Upgradeable Loader"        , "BPFLoaderUpgradeab1e11111111111111111111111"),
        ("BPF Loader 2"                  , "BPFLoader2111111111111111111111111111111111"),
        ("Config Program"                , "Config1111111111111111111111111111111111111"),
        ("Feature Program"               , "Feature111111111111111111111111111111111111"),
        ("Native Loader"                 , "NativeLoader1111111111111111111111111111111"),
        ("Stake Program"                 , "Stake11111111111111111111111111111111111111"),
        ("Sysvar"                        , "Sysvar1111111111111111111111111111111111111"),
        ("Vote Program"                  , "Vote111111111111111111111111111111111111111"),
        ("Stake Config"                  , "StakeConfig11111111111111111111111111111111"),
        ("Sol Program"                   , "So11111111111111111111111111111111111111112"),
        ("Clock Sysvar"                  , "SysvarC1ock11111111111111111111111111111111"),
        ("Epoch Schedule Sysvar"         , "SysvarEpochSchedu1e111111111111111111111111"),
        ("Fees Sysvar"                   , "SysvarFees111111111111111111111111111111111"),
        ("Last Restart Slog Sysvar"      , "SysvarLastRestartS1ot1111111111111111111111"),
        ("Recent Blockhashes Sysvar"     , "SysvarRecentB1ockHashes11111111111111111111"),
        ("Rent Sysvar"                   , "SysvarRent111111111111111111111111111111111"),
        ("Slot Hashes"                   , "SysvarS1otHashes111111111111111111111111111"),
        ("Slot History"                  , "SysvarS1otHistory11111111111111111111111111"),
        ("Stake History"                 , "SysvarStakeHistory1111111111111111111111111"),
        ("MagicBlock System Program"     , "Magic11111111111111111111111111111111111111"),
        ("MagicBlock Delegation Program" , "DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh"),
        ("Luzid Authority"               , "LUzidNSiPNjYNkxZcUm5hYHwnWPwsUfh2US1cpWwaBm"),
    ]
    .into_iter()
    .map(|(name, key)| (Pubkey::from_str(key).unwrap(), name))
    .collect();
}

pub fn map_instruction(
    instruction: &impl ParseableInstruction,
    idl: Option<&Idl>,
    deserializer: &impl ChainparserDeserialize,
) -> Result<InstructionMapResult, ChainparserError> {
    //  let program_id = instruction.program_id();
    let program_name = idl.as_ref().map(|idl| idl.name.to_string());

    // Step 1: Find the best matching IDL instruction

    let idl_instruction = idl
        .as_ref()
        .and_then(|idl| find_best_matching_idl_ix(&idl.instructions, instruction))
        .ok_or_else(|| {
            ChainparserError::UnsupportedDeserializer("no matching ix found".to_string())
        })?;

    // Step 2: Decode the instruction data
    let decoded_args = decode_instruction_data(
        &idl_instruction,
        instruction.data(),
        deserializer,
        idl.unwrap(),
    )?;
    /*
    /// Then it finds the best matching IDL instruction for provided instruction and
    /// creates an entry for each account pubkey providing its name.
    /// */
    let mapper = idl
        .as_ref()
        .and_then(|idl| InstructionMapper::determine_accounts_mapper(instruction, idl));
    let program_name = idl.as_ref().map(|idl| idl.name.to_string());
    let program_id = instruction.program_id();

    let mut accounts = HashMap::new();
    let mut instruction_name = None::<String>;
    let ix_accounts = instruction.accounts();
    for (idx, pubkey) in ix_accounts.into_iter().enumerate() {
        if let Some(program_name) = program_name.as_ref() {
            if &pubkey == program_id {
                accounts.insert(pubkey, program_name.to_string());
                continue;
            }
        }
        if let Some(mapper) = &mapper {
            let name = mapper
                .idl_instruction
                .accounts
                .get(idx)
                .map(|x| x.name().to_string());
            if let Some(name) = name {
                accounts.insert(pubkey, name);
            }
            instruction_name.replace(mapper.idl_instruction.name.to_string());
        }
    }
    let program_name = idl.map(|x| x.name.to_string());

    // Step 4: Return the result
    Ok(InstructionMapResult {
        accounts,
        instruction_name: Some(idl_instruction.name.clone()),
        program_name,
        decoded_args,
    })
}

pub struct InstructionMapper {
    idl_instruction: IdlInstruction,
}

pub struct InstructionMapResult {
    pub accounts: HashMap<Pubkey, String>,
    pub instruction_name: Option<String>,
    pub program_name: Option<String>,
    pub decoded_args: serde_json::Value,
}

impl InstructionMapper {
    fn determine_accounts_mapper(
        instruction: &impl ParseableInstruction,
        idl: &Idl,
    ) -> Option<InstructionMapper> {
        find_best_matching_idl_ix(&idl.instructions, instruction)
            .map(|idl_instruction| InstructionMapper { idl_instruction })
    }
}

pub fn find_best_matching_idl_ix(
    ix_idls: &[IdlInstruction],
    ix: &impl ParseableInstruction,
) -> Option<IdlInstruction> {
    let mut best_match = None;
    let mut best_match_score = 0;
    for idl_ix in ix_idls {
        let disc = discriminator_from_ix(idl_ix);
        trace!("Discriminator for '{}': {:?}", idl_ix.name, disc);
        if disc.len() > ix.data().len() {
            continue;
        }
        let mut score = 0;
        for (a, b) in disc.iter().zip(ix.data()) {
            if a != b {
                break;
            }
            score += 1;
        }
        if score > best_match_score {
            best_match = Some(idl_ix);
            best_match_score = score;
        }
    }
    best_match.cloned()
}
pub fn decode_instruction_data(
    idl_instruction: &IdlInstruction,
    data: &[u8],
    deserializer: &impl ChainparserDeserialize,
    idl: &Idl,
) -> ChainparserResult<Value> {
    // Step 1: Extract and verify the discriminator
    let disc = discriminator_from_ix(idl_instruction);
    trace!("Expected Discriminator: {:?}", disc);
    trace!("Actual Instruction Data: {:?}", &data[..disc.len()]);

    if !data.starts_with(&disc) {
        print!("discriminator mismatch")
        //return Err(ChainparserError::DiscriminatorMismatch);
    }

    // Step 2: Deserialize the remaining data
    let mut buf = &data[disc.len()..];
    let mut decoded_args = Map::new();

    for arg in &idl_instruction.args {
        // println!("Decoding argument: {}", arg.name);
        let value = deserialize_value(&arg.ty, &mut buf, deserializer, idl)?;
        decoded_args.insert(arg.name.clone(), value);
    }

    Ok(Value::Object(decoded_args))
} // so how to update this then
fn deserialize_value(
    ty: &IdlType,
    buf: &mut &[u8],
    deserializer: &impl ChainparserDeserialize,
    idl: &Idl,
) -> ChainparserResult<Value> {
    match ty {
        IdlType::U8 => deserializer
            .u8(buf)
            .map(|v| Value::Number(v.into()))
            .map_err(|e| e.into()),
        IdlType::U16 => deserializer
            .u16(buf)
            .map(|v| Value::Number(v.into()))
            .map_err(|e| e.into()),
        IdlType::U32 => deserializer
            .u32(buf)
            .map(|v| Value::Number(v.into()))
            .map_err(|e| e.into()),
        IdlType::U64 => deserializer
            .u64(buf)
            .map(|v| Value::Number(v.into()))
            .map_err(|e| e.into()),
        IdlType::Bool => deserializer
            .bool(buf)
            .map(|v| Value::Bool(v))
            .map_err(|e| e.into()),
        IdlType::String => deserializer
            .string(buf)
            .map(Value::String)
            .map_err(|e| e.into()),
        IdlType::Bytes => {
            let bytes = deserializer.bytes(buf)?;
            Ok(Value::Array(
                bytes.into_iter().map(|b| Value::Number(b.into())).collect(),
            ))
        }
        IdlType::Option(inner_ty) => {
            let is_some = deserializer.option(buf)?;
            if is_some {
                deserialize_value(inner_ty, buf, deserializer, idl)
            } else {
                Ok(Value::Null)
            }
        }
        IdlType::PublicKey => deserializer
            .pubkey(buf)
            .map(|pk| Value::String(pk.to_string()))
            .map_err(|e| e.into()),
        IdlType::Vec(inner_ty) => {
            let len = deserializer.u32(buf)? as usize;
            let mut values = Vec::new();
            for _ in 0..len {
                values.push(deserialize_value(inner_ty, buf, deserializer, idl)?);
            }
            Ok(Value::Array(values))
        }
        IdlType::Defined(type_name) => deserialize_defined_type(type_name, idl, buf, deserializer),
        _ => Err(ChainparserError::UnsupportedDeserializer(format!(
            "{:?}",
            ty
        ))),
    }
}
fn deserialize_defined_type(
    type_name: &str,
    idl: &Idl,
    buf: &mut &[u8],
    deserializer: &impl ChainparserDeserialize,
) -> ChainparserResult<Value> {
    let type_def = idl
        .types
        .iter()
        .find(|t| t.name == type_name)
        .ok_or_else(|| ChainparserError::CannotFindDefinedType(type_name.to_string()))?;

    match &type_def.ty {
        IdlTypeDefinitionTy::Struct { fields } => {
            let mut decoded_fields = Map::new();
            for field in fields {
                //   println!("Decoding nested field: {}", field.name);

                // Set the current field name
                //   set_current_field_name(&field.name);

                // Decode the field value
                let value = deserialize_value(&field.ty, buf, deserializer, idl)?;

                // Insert the decoded value into the result
                decoded_fields.insert(field.name.clone(), value);

                // Clear the current field name
                //   set_current_field_name("");
            }
            Ok(Value::Object(decoded_fields))
        }
        IdlTypeDefinitionTy::Enum { variants } => {
            let discriminant = deserializer.u8(buf)?;
            let variant = variants
                .get(discriminant as usize)
                .ok_or_else(|| ChainparserError::InvalidEnumVariantDiscriminator(discriminant))?;

            let mut decoded_variant = Map::new();
            if let Some(EnumFields::Named(fields)) = &variant.fields {
                for field in fields {
                    let value = deserialize_value(&field.ty, buf, deserializer, idl)?;
                    decoded_variant.insert(field.name.clone(), value);
                }
            }

            Ok(Value::Object(Map::from_iter([(
                variant.name.clone(),
                Value::Object(decoded_variant),
            )])))
        }
    }
}
