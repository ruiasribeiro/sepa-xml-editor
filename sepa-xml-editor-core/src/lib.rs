use wasm_bindgen::prelude::*;

// Some helpful documentation about the SEPA Payments XML format:
// https://docs.oracle.com/cd/E16582_01/doc.91/e15104/fields_sepa_pay_file_appx.htm#EOAEL01692

#[wasm_bindgen]
pub fn parse_document(document: &str) -> Result<Document, wasm_bindgen::JsError> {
    Ok(quick_xml::de::from_str(document)?)
}

#[wasm_bindgen]
pub fn write_xml(document: &Document) -> Result<String, wasm_bindgen::JsError> {
    let mut buffer = Vec::new();
    let mut writer = quick_xml::Writer::new_with_indent(&mut buffer, b' ', 4);

    writer.write_serializable("Document", &document)?;

    Ok(String::from_utf8(buffer)?)
}

#[wasm_bindgen]
pub fn edit_headers(
    document: &mut Document,
    id: String,
    creation_date: String,
    initiating_party_name: String,
) {
    let header = &mut document.customer_credit_transfer_initiation.header;

    header.id = id;
    header.creation_date = creation_date;
    header.initiating_party.name = initiating_party_name;
}

#[wasm_bindgen]
pub fn recompute_headers(document: &mut Document) -> Result<(), wasm_bindgen::JsError> {
    let number_of_transactions = document
        .customer_credit_transfer_initiation
        .payment_information
        .transactions
        .len();

    let mut control_sum = 0_f64;

    for transaction in document
        .customer_credit_transfer_initiation
        .payment_information
        .transactions
        .iter()
    {
        control_sum += transaction.amount.instructed_amount.value.parse::<f64>()?;
    }

    document
        .customer_credit_transfer_initiation
        .header
        .number_of_transactions = number_of_transactions;

    document
        .customer_credit_transfer_initiation
        .header
        .control_sum = control_sum.to_string();

    Ok(())
}

#[wasm_bindgen]
pub fn add_transaction(
    document: &mut Document,
    name: String,
    iban: String,
    bic: String,
    amount: String,
) {
    let transaction = CreditorTransferTransactionInformation {
        payment_identification: PaymentIdentification {
            end_to_end_id: "NOTPROVIDED".into(),
        },
        amount: Amount {
            instructed_amount: InstructedAmount {
                currency: "EUR".into(),
                value: amount,
            },
        },
        creditor_agent: CreditorAgent {
            financial_institution_id: FinancialInstitutionId { bic },
        },
        creditor: Creditor { name },
        creditor_account: CreditorAccount {
            id: CreditorAccountId { iban },
        },
    };

    document
        .customer_credit_transfer_initiation
        .payment_information
        .transactions
        .push(transaction);
}

#[wasm_bindgen]
pub fn edit_transaction(
    document: &mut Document,
    transaction_index: usize,
    name: String,
    iban: String,
    bic: String,
    amount: String,
) -> Result<(), wasm_bindgen::JsError> {
    let transaction = document
        .customer_credit_transfer_initiation
        .payment_information
        .transactions
        .get_mut(transaction_index)
        .ok_or(wasm_bindgen::JsError::new("could not get transaction"))?;

    transaction.creditor.name = name;
    transaction.creditor_account.id.iban = iban;
    transaction.creditor_agent.financial_institution_id.bic = bic;
    transaction.amount.instructed_amount.value = amount;

    Ok(())
}

#[wasm_bindgen]
pub fn remove_transaction(
    document: &mut Document,
    transaction_index: usize,
) -> Result<(), wasm_bindgen::JsError> {
    if transaction_index
        >= document
            .customer_credit_transfer_initiation
            .payment_information
            .transactions
            .len()
    {
        return Err(wasm_bindgen::JsError::new("invalid index"));
    }

    document
        .customer_credit_transfer_initiation
        .payment_information
        .transactions
        .remove(transaction_index);

    Ok(())
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct Document {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,

    #[serde(rename = "@xmlns:xsi")]
    pub xmlns_xsi: String,

    #[serde(rename = "CstmrCdtTrfInitn")]
    pub customer_credit_transfer_initiation: CustomerCreditTransferInitiation,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct CustomerCreditTransferInitiation {
    #[serde(rename = "GrpHdr")]
    pub header: GroupHeader,

    #[serde(rename = "PmtInf")]
    pub payment_information: PaymentInformation,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct GroupHeader {
    #[serde(rename = "MsgId")]
    pub id: String,

    #[serde(rename = "CreDtTm")] //, with = "my_date_format")]
    pub creation_date: String, //chrono::DateTime<chrono::Utc>,

    #[serde(rename = "NbOfTxs")]
    pub number_of_transactions: usize,

    #[serde(rename = "CtrlSum")]
    pub control_sum: String,

    #[serde(rename = "InitgPty")]
    pub initiating_party: InitiatingParty,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct InitiatingParty {
    #[serde(rename = "Nm")]
    pub name: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct PaymentInformation {
    #[serde(rename = "PmtInfId")]
    pub id: String,

    #[serde(rename = "PmtMtd")]
    pub method: String, // Hardcoded to "TRF".

    #[serde(rename = "NbOfTxs")]
    pub number_of_transactions: i32,

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
#[wasm_bindgen(getter_with_clone)]
pub struct PaymentTypeInformation {
    #[serde(rename = "CtgyPurp")]
    pub category_purpose: CategoryPurposeCode,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct CategoryPurposeCode {
    #[serde(rename = "Cd")]
    pub code: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct Debtor {
    #[serde(rename = "Nm")]
    pub name: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct DebtorAccount {
    #[serde(rename = "Id")]
    pub id: DebtorAccountId,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct DebtorAccountId {
    #[serde(rename = "IBAN")]
    pub iban: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct DebtorAgent {
    #[serde(rename = "FinInstnId")]
    pub financial_institution_id: FinancialInstitutionId,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct FinancialInstitutionId {
    #[serde(rename = "BIC")]
    pub bic: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
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
#[wasm_bindgen(getter_with_clone)]
pub struct PaymentIdentification {
    #[serde(rename = "EndToEndId")]
    pub end_to_end_id: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct Amount {
    #[serde(rename = "InstdAmt")]
    pub instructed_amount: InstructedAmount,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct InstructedAmount {
    #[serde(rename = "@Ccy")]
    pub currency: String,

    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct Creditor {
    #[serde(rename = "Nm")]
    pub name: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct CreditorAgent {
    #[serde(rename = "FinInstnId")]
    pub financial_institution_id: FinancialInstitutionId,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct CreditorAccount {
    #[serde(rename = "Id")]
    pub id: CreditorAccountId,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct CreditorAccountId {
    #[serde(rename = "IBAN")]
    pub iban: String,
}
