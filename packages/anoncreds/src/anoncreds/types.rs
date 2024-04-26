//use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Record};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    convert::{TryFrom, TryInto},
};

pub use ursa::{
    bn::BigNumber,
    cl::{
        AggregatedProof, CredentialPrimaryPublicKey, CredentialPublicKey,
        CredentialRevocationPublicKey, CredentialSchema, CredentialSchemaBuilder,
        NonCredentialSchema, NonCredentialSchemaBuilder, NonRevocProof, NonRevocProofCList,
        NonRevocProofXList, Predicate, PredicateType, PrimaryEqualProof,
        PrimaryPredicateInequalityProof, PrimaryProof, Proof, RevocationKeyPublic,
        RevocationRegistry, SubProof, SubProofRequest,
    },
    errors::UrsaCryptoError,
    pair::{GroupOrderElement, Pair, PointG1, PointG2},
};

use crate::anoncreds::errors::TypeConversionError;

pub enum AnonCredsObjects {
    RevocationRegistry(RevocationRegistry),
    PointG1Bytes(PointG1Bytes),
}

pub type ConversionResult<T> = Result<T, TypeConversionError>;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct BigNumberBytes(pub String);

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct PointG1Bytes(Binary);

impl PointG1Bytes {
    pub fn to_point_g1(self) -> ConversionResult<PointG1> {
        PointG1::from_bytes(self.0.as_slice()).map_err(|e| e.into())
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct PointG2Bytes(Binary);

impl PointG2Bytes {
    pub fn to_point_g2(self) -> ConversionResult<PointG2> {
        PointG2::from_bytes(self.0.as_slice()).map_err(|e| e.into())
    }
}

impl TryFrom<BigNumberBytes> for BigNumber {
    type Error = TypeConversionError;
    fn try_from(value: BigNumberBytes) -> Result<Self, Self::Error> {
        Ok(BigNumber::from_dec(&value.0)?)
    }
}

impl TryFrom<BigNumber> for BigNumberBytes {
    type Error = TypeConversionError;
    fn try_from(value: BigNumber) -> Result<Self, Self::Error> {
        let bns = value.to_dec()?;
        Ok(BigNumberBytes(bns))
    }
}

impl TryFrom<&BigNumber> for BigNumberBytes {
    type Error = TypeConversionError;
    fn try_from(value: &BigNumber) -> Result<Self, Self::Error> {
        let bns = value.to_dec()?;
        Ok(BigNumberBytes(bns))
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WBTreeSet<T: Ord>(Vec<T>);

impl<T: Ord> TryFrom<WBTreeSet<T>> for BTreeSet<T> {
    type Error = TypeConversionError;

    fn try_from(value: WBTreeSet<T>) -> Result<Self, Self::Error> {
        let mut set = BTreeSet::new();
        for a in value.0 {
            set.insert(a);
        }
        Ok(set)
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WMap(pub Vec<Record<BigNumberBytes>>);

impl TryFrom<WMap> for BTreeMap<String, BigNumber> {
    type Error = TypeConversionError;

    fn try_from(value: WMap) -> Result<Self, Self::Error> {
        let mut map = BTreeMap::new();
        for e in value.0 {
            let s = String::from_utf8(e.0)
                .map_err(|e| TypeConversionError::Conversion(e.to_string()))?;
            let bn = e.1.try_into()?;
            map.insert(s, bn);
        }
        Ok(map)
    }
}

impl TryFrom<WMap> for HashMap<String, BigNumber> {
    type Error = TypeConversionError;

    fn try_from(value: WMap) -> Result<Self, Self::Error> {
        let mut map = HashMap::new();
        for e in value.0 {
            let s = String::from_utf8(e.0)
                .map_err(|e| TypeConversionError::Conversion(e.to_string()))?;
            let bn = e.1.try_into()?;
            map.insert(s, bn);
        }
        Ok(map)
    }
}

impl TryFrom<HashMap<String, BigNumber>> for WMap {
    type Error = TypeConversionError;

    fn try_from(value: HashMap<String, BigNumber>) -> Result<Self, Self::Error> {
        let mut rec: Vec<Record<BigNumberBytes>> = Vec::new();
        for (k, v) in value.iter() {
            let r = (k.as_bytes().to_vec(), BigNumberBytes::try_from(v)?);
            rec.push(r);
        }
        Ok(WMap(rec))
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WGroupOrderElement {
    // amcl::bn254::bn::Big has `from_hex` string
    // called by GroupOrderElement::from_string()
    // Converts to amcl::bn254::bn::Big
    bn_hex: String,
}

impl TryFrom<&WGroupOrderElement> for GroupOrderElement {
    type Error = TypeConversionError;

    fn try_from(value: &WGroupOrderElement) -> Result<Self, Self::Error> {
        GroupOrderElement::from_string(&value.bn_hex).map_err(|e| e.into())
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WProof {
    pub proofs: Vec<WSubProof>,
    pub aggregated_proof: WAggregatedProof,
}

impl WProof {
    pub fn mock() -> Self {
        Self {
            proofs: vec![],
            aggregated_proof: WAggregatedProof {
                c_hash: BigNumberBytes("c_hash".into()),
                c_list: vec![],
            },
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WAggregatedProof {
    pub c_hash: BigNumberBytes,
    pub c_list: Vec<Vec<u8>>,
}

impl TryFrom<WProof> for Proof {
    type Error = TypeConversionError;
    fn try_from(value: WProof) -> Result<Self, Self::Error> {
        Ok(Proof {
            proofs: value
                .proofs
                .iter()
                .map(|e| {
                    e.clone()
                        .try_into()
                        .map_err(|_| TypeConversionError::Conversion("Proofs value".to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?,
            aggregated_proof: AggregatedProof {
                c_hash: value.aggregated_proof.c_hash.try_into()?,
                c_list: value.aggregated_proof.c_list,
            },
        })
    }
}

/// SubProofRequest type from Libursa
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WSubProofReq {
    pub revealed_attrs: WBTreeSet<String>,
    pub predicates: WBTreeSet<WPredicate>,
}

impl TryFrom<WSubProofReq> for SubProofRequest {
    type Error = TypeConversionError;

    fn try_from(value: WSubProofReq) -> Result<Self, Self::Error> {
        let revealed_attrs = value.revealed_attrs.try_into()?;
        let p_vec = value
            .predicates
            .0
            .iter()
            .map(|e| e.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        let p_w_btreeset = WBTreeSet(p_vec);
        let predicates = p_w_btreeset.try_into()?;
        Ok(SubProofRequest {
            revealed_attrs,
            predicates,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub struct WPredicate {
    pub attr_name: String,
    pub p_type: WPredicateType,
    pub value: u32,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum WPredicateType {
    GE,
    LE,
    GT,
    LT,
}

impl TryFrom<&WPredicate> for Predicate {
    type Error = TypeConversionError;

    fn try_from(value: &WPredicate) -> Result<Self, Self::Error> {
        let pt = match value.p_type {
            WPredicateType::GE => PredicateType::GE,
            WPredicateType::LE => PredicateType::LE,
            WPredicateType::GT => PredicateType::GT,
            WPredicateType::LT => PredicateType::LT,
        };

        Ok(Predicate {
            attr_name: value.attr_name.clone(),
            p_type: pt,
            value: value.value as i32,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WCredentialSchema {
    pub attrs: WBTreeSet<String>, // BTreeSet<String>
}

impl TryFrom<WCredentialSchema> for CredentialSchema {
    type Error = TypeConversionError;
    fn try_from(value: WCredentialSchema) -> Result<Self, Self::Error> {
        let mut builder = CredentialSchemaBuilder::new()?;
        for a in &value.attrs.0 {
            builder.add_attr(a)?;
        }
        Ok(builder.finalize()?)
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WNonCredentialSchema {
    pub attrs: WBTreeSet<String>, // BTreeSet<String>
}

impl TryFrom<WNonCredentialSchema> for NonCredentialSchema {
    type Error = TypeConversionError;
    fn try_from(value: WNonCredentialSchema) -> Result<Self, Self::Error> {
        let mut builder = NonCredentialSchemaBuilder::new()?;
        for a in &value.attrs.0 {
            builder.add_attr(a)?;
        }
        Ok(builder.finalize()?)
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WCredentialPubKey {
    pub p_key: WCredentialPrimaryPubKey,
    pub r_key: Option<WCredentialRevocationPubKey>,
}

impl TryFrom<WCredentialPubKey> for CredentialPublicKey {
    type Error = TypeConversionError;

    fn try_from(value: WCredentialPubKey) -> Result<Self, Self::Error> {
        if value.r_key.is_some() {
            Err(TypeConversionError::Conversion(String::from(
                "revocation not supported",
            )))
        } else {
            Ok(CredentialPublicKey {
                p_key: value.p_key.try_into()?,
                r_key: None,
            })
        }
    }
}

impl TryFrom<CredentialPublicKey> for WCredentialPubKey {
    type Error = TypeConversionError;

    fn try_from(value: CredentialPublicKey) -> Result<Self, Self::Error> {
        if value.r_key.is_some() {
            Err(TypeConversionError::Conversion(String::from(
                "revocation not supported",
            )))
        } else {
            Ok(WCredentialPubKey {
                p_key: value.p_key.try_into()?,
                r_key: None,
            })
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WCredentialPrimaryPubKey {
    n: BigNumberBytes,
    s: BigNumberBytes,
    // https://docs.rs/cosmwasm-std/latest/cosmwasm_std/type.Record.html#
    // Record key is also binary
    // We cannot set it as a type because this is the req key value,
    // it is dependent on the schema,
    // underneath it is a `Record`, a vec
    r: WMap,
    rctxt: BigNumberBytes,
    z: BigNumberBytes,
}

impl TryFrom<WCredentialPrimaryPubKey> for CredentialPrimaryPublicKey {
    type Error = TypeConversionError;
    fn try_from(v: WCredentialPrimaryPubKey) -> Result<Self, Self::Error> {
        Ok(CredentialPrimaryPublicKey {
            n: v.n.try_into()?,
            s: v.s.try_into()?,
            r: v.r.try_into()?,
            rctxt: v.rctxt.try_into()?,
            z: v.z.try_into()?,
        })
    }
}

impl TryFrom<CredentialPrimaryPublicKey> for WCredentialPrimaryPubKey {
    type Error = TypeConversionError;
    fn try_from(v: CredentialPrimaryPublicKey) -> Result<Self, Self::Error> {
        Ok(WCredentialPrimaryPubKey {
            n: v.n.try_into()?,
            s: v.s.try_into()?,
            r: v.r.try_into()?,
            rctxt: v.rctxt.try_into()?,
            z: v.z.try_into()?,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WCredentialRevocationPubKey {
    g: PointG1Bytes,
    g_dash: PointG2Bytes,
    h: PointG1Bytes,
    h0: PointG1Bytes,
    h1: PointG1Bytes,
    h2: PointG1Bytes,
    htilde: PointG1Bytes,
    h_cap: PointG2Bytes,
    u: PointG2Bytes,
    pk: PointG1Bytes,
    y: PointG2Bytes,
}

impl TryFrom<WCredentialRevocationPubKey> for CredentialRevocationPublicKey {
    type Error = TypeConversionError;

    fn try_from(value: WCredentialRevocationPubKey) -> Result<Self, Self::Error> {
        Ok(CredentialRevocationPublicKey {
            g: value.g.to_point_g1()?,
            g_dash: value.g_dash.to_point_g2()?,
            h: value.h.to_point_g1()?,
            h0: value.h0.to_point_g1()?,
            h1: value.h1.to_point_g1()?,
            h2: value.h2.to_point_g1()?,
            htilde: value.htilde.to_point_g1()?,
            h_cap: value.h_cap.to_point_g2()?,
            u: value.u.to_point_g2()?,
            pk: value.pk.to_point_g1()?,
            y: value.y.to_point_g2()?,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WRevocationKeyPublic {
    // Hex String from Pair
    pub pair: String,
}

impl TryFrom<WRevocationKeyPublic> for RevocationKeyPublic {
    type Error = TypeConversionError;
    fn try_from(value: WRevocationKeyPublic) -> Result<Self, Self::Error> {
        let z = Pair::from_string(&value.pair).map_err::<TypeConversionError, _>(|e| e.into())?;
        Ok(RevocationKeyPublic { z })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WRevocationRegistry {
    // the Accumulator type is alias for PointG2
    pub accum: Binary,
}

impl TryFrom<WRevocationRegistry> for RevocationRegistry {
    type Error = TypeConversionError;

    fn try_from(value: WRevocationRegistry) -> Result<Self, Self::Error> {
        let accum = PointG2::from_bytes(value.accum.as_slice())
            .map_err::<TypeConversionError, _>(|e| e.into())?;
        Ok(RevocationRegistry { accum })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WSubProof {
    pub primary_proof: WPrimaryProof,
    pub non_revoc_proof: Option<WNonRevocProof>,
}

impl TryFrom<WSubProof> for SubProof {
    type Error = TypeConversionError;
    fn try_from(value: WSubProof) -> Result<Self, Self::Error> {
        let n = match value.non_revoc_proof {
            Some(non) => Some(non.try_into()?),
            None => None,
        };
        Ok(SubProof {
            primary_proof: value.primary_proof.try_into()?,
            non_revoc_proof: n,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WPrimaryProof {
    pub eq_proof: WPrimaryEqualProof,
    pub ne_proofs: Vec<WPrimaryPredicateInequalityProof>,
}

impl TryFrom<WPrimaryProof> for PrimaryProof {
    type Error = TypeConversionError;
    fn try_from(value: WPrimaryProof) -> Result<Self, Self::Error> {
        let ne_proofs = value
            .ne_proofs
            .iter()
            .map(|e| e.clone().try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PrimaryProof {
            eq_proof: value.eq_proof.try_into()?,
            ne_proofs,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WPrimaryEqualProof {
    revealed_attrs: WMap,
    a_prime: BigNumberBytes,
    e: BigNumberBytes,
    v: BigNumberBytes,
    m: WMap, // HashMap<String /* attr_name of all except revealed */, BigNumber>,
    m2: BigNumberBytes,
}

impl TryFrom<WPrimaryEqualProof> for PrimaryEqualProof {
    type Error = TypeConversionError;
    fn try_from(v: WPrimaryEqualProof) -> Result<Self, Self::Error> {
        Ok(PrimaryEqualProof {
            revealed_attrs: v.revealed_attrs.try_into()?,
            a_prime: v.a_prime.try_into()?,
            e: v.e.try_into()?,
            v: v.v.try_into()?,
            m: v.m.try_into()?,
            m2: v.m2.try_into()?,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WPrimaryPredicateInequalityProof {
    u: WMap, // HashMap<String, BigNumber>,
    r: WMap, // HashMap<String, BigNumber>,
    mj: BigNumberBytes,
    alpha: BigNumberBytes,
    t: WMap, // HashMap<String, BigNumber>,
    predicate: WPredicate,
}

impl TryFrom<WPrimaryPredicateInequalityProof> for PrimaryPredicateInequalityProof {
    type Error = TypeConversionError;
    fn try_from(v: WPrimaryPredicateInequalityProof) -> Result<Self, Self::Error> {
        Ok(PrimaryPredicateInequalityProof {
            u: v.u.try_into()?,
            r: v.r.try_into()?,
            mj: v.mj.try_into()?,
            alpha: v.alpha.try_into()?,
            t: v.t.try_into()?,
            predicate: Predicate::try_from(&v.predicate)?,
        })
    }
}

/// Converts to NonRevocProof
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WNonRevocProof {
    /// Converts to NonRevocProofXList
    pub x_list: Vec<WGroupOrderElement>,
    /// Converts to NonRevocProofCList,
    pub c_list: (Vec<PointG1Bytes>, Vec<PointG2Bytes>),
}

impl TryFrom<WNonRevocProof> for NonRevocProof {
    type Error = TypeConversionError;

    fn try_from(value: WNonRevocProof) -> Result<Self, Self::Error> {
        let x_list_ge = value
            .x_list
            .iter()
            .map(|e| e.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        let c_list_p1 = value
            .c_list
            .0
            .iter()
            .map(|e| e.to_owned().to_point_g1())
            .collect::<Result<Vec<_>, _>>()?;
        let c_list_p2 = value
            .c_list
            .1
            .iter()
            .map(|e| e.to_owned().to_point_g2())
            .collect::<Result<Vec<_>, _>>()?;

        let c_list_struct = NonRevocProofCList {
            e: c_list_p1[0],
            d: c_list_p1[1],
            a: c_list_p1[2],
            g: c_list_p1[3],
            w: c_list_p2[0],
            s: c_list_p2[1],
            u: c_list_p2[2],
        };

        Ok(NonRevocProof {
            x_list: NonRevocProofXList::from_list(&x_list_ge),
            c_list: c_list_struct,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WSubProofReqParams {
    pub sub_proof_request: WSubProofReq,
    pub credential_schema: WCredentialSchema,
    pub non_credential_schema: WNonCredentialSchema,
    pub credential_pub_key: WCredentialPubKey,
    pub rev_key_pub: Option<WRevocationKeyPublic>,
    pub rev_reg: Option<WRevocationRegistry>,
}

#[derive(Serialize, Deserialize)]
pub struct SubProofReqParams {
    pub sub_proof_request: SubProofRequest,
    pub credential_schema: CredentialSchema,
    pub non_credential_schema: NonCredentialSchema,
    pub credential_pub_key: WCredentialPubKey,
    pub rev_key_pub: Option<RevocationKeyPublic>,
    pub rev_reg: Option<RevocationRegistry>,
}

impl TryFrom<WSubProofReqParams> for SubProofReqParams {
    type Error = TypeConversionError;

    fn try_from(v: WSubProofReqParams) -> Result<Self, Self::Error> {
        let mut rev_key_pub: Option<RevocationKeyPublic> = None;
        let mut rev_reg: Option<RevocationRegistry> = None;

        if let (Some(kp), Some(reg)) = (v.rev_key_pub, v.rev_reg) {
            let r: RevocationRegistry = reg.try_into()?;
            let k: RevocationKeyPublic = kp.try_into()?;
            rev_key_pub = Some(k);
            rev_reg = Some(r);
        }

        Ok(SubProofReqParams {
            sub_proof_request: v.sub_proof_request.try_into()?,
            credential_schema: v.credential_schema.try_into()?,
            non_credential_schema: v.non_credential_schema.try_into()?,
            credential_pub_key: v.credential_pub_key,
            rev_key_pub,
            rev_reg,
        })
    }
}
