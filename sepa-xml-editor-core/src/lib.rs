use wasm_bindgen::prelude::*;

// Some helpful documentation about the SEPA Payments XML format:
// https://docs.oracle.com/cd/E16582_01/doc.91/e15104/fields_sepa_pay_file_appx.htm#EOAEL01692

#[wasm_bindgen]
pub fn parse_document(document: &str) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsError> {
    let document: Document = quick_xml::de::from_str(document)?;
    Ok(serde_wasm_bindgen::to_value(&document)?)
}

#[wasm_bindgen]
pub fn write_xml(document: wasm_bindgen::JsValue) -> Result<String, wasm_bindgen::JsError> {
    let document: Document = serde_wasm_bindgen::from_value(document)?;

    let mut buffer = Vec::new();
    let mut writer = quick_xml::Writer::new_with_indent(&mut buffer, b' ', 4);

    writer.write_serializable("Document", &document)?;

    Ok(String::from_utf8(buffer)?)
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Document {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,

    #[serde(rename = "@xmlns:xsi")]
    pub xmlns_xsi: String,

    #[serde(rename = "CstmrCdtTrfInitn")]
    pub customer_credit_transfer_initiation: CustomerCreditTransferInitiation,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CustomerCreditTransferInitiation {
    #[serde(rename = "GrpHdr")]
    pub header: GroupHeader,

    #[serde(rename = "PmtInf")]
    pub payment_information: PaymentInformation,
}

pub mod my_date_format {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GroupHeader {
    #[serde(rename = "MsgId")]
    pub id: String,

    #[serde(rename = "CreDtTm", with = "my_date_format")]
    pub creation_date: chrono::DateTime<chrono::Utc>,

    #[serde(rename = "NbOfTxs")]
    pub number_of_transactions: usize,

    #[serde(rename = "CtrlSum")]
    pub control_sum: String,

    #[serde(rename = "InitgPty")]
    pub initiating_party: InitiatingParty,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct InitiatingParty {
    #[serde(rename = "Nm")]
    pub name: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PaymentInformation {
    #[serde(rename = "PmtInfId")]
    pub id: String,

    #[serde(rename = "PmtMtd")]
    pub method: String, // Hardcoded to "TRF".

    #[serde(rename = "NbOfTxs")]
    pub number_of_transactions: usize,

    #[serde(rename = "CtrlSum")]
    pub control_sum: String,

    #[serde(rename = "PmtTpInf")]
    pub type_information: PaymentTypeInformation,

    #[serde(rename = "ReqdExctnDt")]
    pub requested_execution_date: String, //chrono::NaiveDate,

    #[serde(rename = "Dbtr")]
    pub debtor: Debtor,

    #[serde(rename = "DbtrAcct")]
    pub debtor_account: DebtorAccount,

    #[serde(rename = "DbtrAgt")]
    pub debtor_agent: DebtorAgent,

    #[serde(rename = "CdtTrfTxInf")]
    pub transactions: Vec<CreditorTransferTransactionInformation>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PaymentTypeInformation {
    #[serde(rename = "CtgyPurp")]
    pub category_purpose: CategoryPurposeCode,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CategoryPurposeCode {
    #[serde(rename = "Cd")]
    pub code: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Debtor {
    #[serde(rename = "Nm")]
    pub name: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DebtorAccount {
    #[serde(rename = "Id")]
    pub id: DebtorAccountId,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DebtorAccountId {
    #[serde(rename = "IBAN")]
    pub iban: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DebtorAgent {
    #[serde(rename = "FinInstnId")]
    pub financial_institution_id: FinancialInstitutionId,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct FinancialInstitutionId {
    #[serde(rename = "BIC")]
    pub bic: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreditorTransferTransactionInformation {
    #[serde(rename = "PmtId")]
    pub payment_identification: PaymentIdentification,

    #[serde(rename = "Amt")]
    pub amount: Amount,

    #[serde(rename = "CdtrAgt")]
    pub creditor_agent: CreditorAgent,

    #[serde(rename = "Cdtr")]
    pub creditor: Creditor,

    #[serde(rename = "CdtrAcct")]
    pub creditor_account: CreditorAccount,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PaymentIdentification {
    #[serde(rename = "EndToEndId")]
    pub end_to_end_id: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Amount {
    #[serde(rename = "InstdAmt")]
    pub instructed_amount: InstructedAmount,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct InstructedAmount {
    #[serde(rename = "@Ccy")]
    pub currency: String,

    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Creditor {
    #[serde(rename = "Nm")]
    pub name: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreditorAgent {
    #[serde(rename = "FinInstnId")]
    pub financial_institution_id: FinancialInstitutionId,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreditorAccount {
    #[serde(rename = "Id")]
    pub id: CreditorAccountId,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreditorAccountId {
    #[serde(rename = "IBAN")]
    pub iban: String,
}
