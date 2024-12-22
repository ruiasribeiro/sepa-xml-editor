<script setup lang="ts">
import { computed, onBeforeMount, type Ref, ref } from "vue";

import Button from 'primevue/button';
import Card from 'primevue/card';
import Dialog from 'primevue/dialog';
import FileUpload, { type FileUploadUploaderEvent } from 'primevue/fileupload';
import InputText from 'primevue/inputtext';
import Message from 'primevue/message';

import { Form, type FormSubmitEvent } from '@primevue/forms';

import 'primeicons/primeicons.css'

import init, { parse_document, write_xml } from "../external/sepa-xml-editor-core/sepa_xml_editor_core";
import currency from "currency.js";
import { zodResolver } from "@primevue/forms/resolvers/zod";
import { z } from 'zod';

import { useToast } from 'primevue/usetoast';
import Toast from 'primevue/toast';

import * as ibantools from "ibantools";

const toast = useToast();

type SepaDocument = {
    "@xmlns": string,
    "@xmlns:xsi": string
    CstmrCdtTrfInitn: CustomerCreditTransferInitiation
}


type CustomerCreditTransferInitiation = {
    GrpHdr: GroupHeader,
    PmtInf: PaymentInformation,
}

type GroupHeader = {
    MsgId: string,
    CreDtTm: string,
    NbOfTxs: number,
    CtrlSum: string,
    InitgPty: InitiatingParty
}

type InitiatingParty = {
    Nm: string
}

type PaymentInformation = {
    PmtInfId: string,
    PmtMtd: "TRF",
    NbOfTxs: number,
    CtrlSum: string,
    PmtTpInf: PaymentTypeInformation,
    ReqdExctnDt: string,
    Dbtr: Debtor,
    DbtrAcct: DebtorAccount,
    DbtrAgt: DebtorAgent,
    CdtTrfTxInf: CreditorTransferTransactionInformation[]
}

type PaymentTypeInformation = {
    CtgyPurp: CategoryPurposeCode
}

type CategoryPurposeCode = {
    Cd: string
}

type Debtor = {
    Nm: string,
}

type DebtorAccount = {
    Id: DebtorAccountId
}

type DebtorAccountId = {
    IBAN: string
}

type DebtorAgent = {
    FinInstnId: FinancialInstitutionId
}

type FinancialInstitutionId = {
    BIC: string
}

type CreditorTransferTransactionInformation = {
    PmtId: PaymentIdentification,
    Amt: Amount,
    CdtrAgt: CreditorAgent,
    Cdtr: Creditor,
    CdtrAcct: CreditorAccount
}

type PaymentIdentification = {
    EndToEndId: "NOTPROVIDED"
}

type Amount = {
    InstdAmt: InstructedAmount
}

type InstructedAmount = {
    "@Ccy": "EUR",
    $value: string
}


type Creditor = {
    Nm: string
}

type CreditorAgent = {
    FinInstnId: FinancialInstitutionId
}

type CreditorAccount = {
    Id: CreditorAccountId
}

type CreditorAccountId = {
    IBAN: string
}


onBeforeMount(() => {
    init()
});

const file: Ref<SepaDocument | null, SepaDocument | null> = ref(null);
const transactionToEdit: Ref<number | null, number | null> = ref(null);
const addModalVisible = ref(false);
const editModalVisible = ref(false);
const editHeaderModalVisible = ref(false);

const initialHeaderValues = computed(() => {
    return {
        initiating_party_name: file.value?.CstmrCdtTrfInitn.GrpHdr.InitgPty.Nm,
        id: file.value?.CstmrCdtTrfInitn.GrpHdr.MsgId,
        creation_date: file.value?.CstmrCdtTrfInitn.GrpHdr.CreDtTm
    };
});

function clearFile() {
    file.value = null;
}

function saveFile() {
    if (file.value === null) {
        throw new Error("");
    }

    const fileContent = write_xml(file.value);

    const element = document.createElement('a');
    element.setAttribute('href', 'data:text/xml;charset=utf-8,' + encodeURIComponent(fileContent));
    element.setAttribute('download', `sepa-xml-editor-output_${new Date().toISOString()}.xml`);

    element.style.display = 'none';
    document.body.appendChild(element);

    element.click();

    document.body.removeChild(element);
}

async function onUpload(event: FileUploadUploaderEvent) {
    let uploadedFile;

    if (!Array.isArray(event.files)) {
        uploadedFile = event.files;
    } else {
        uploadedFile = event.files[0];
    }

    file.value = parse_document(await uploadedFile.text())

    console.log(file.value)
    recomputeHeaders()
}

const initialValues = computed(() => {
    if (transactionToEdit.value === null) {
        return {};
    }

    const transaction = file.value?.CstmrCdtTrfInitn.PmtInf.CdtTrfTxInf[transactionToEdit.value];

    return {
        "name": transaction?.Cdtr.Nm,
        "iban": transaction?.CdtrAcct.Id.IBAN,
        "bic": transaction?.CdtrAgt.FinInstnId.BIC,
        "amount": transaction?.Amt.InstdAmt.$value,
    };
});

function editTransaction(event: FormSubmitEvent) {
    if (!event.valid) {
        toast.add({ severity: "error", summary: 'Foram detetados erros na submissão. Por favor corrija e tente novamente.', life: 5000 });
        return;
    }

    const name = event.states.name.value
    const iban = event.states.iban.value
    const bic = event.states.bic.value
    const amount = event.states.amount.value

    if ((file.value === null) || (transactionToEdit.value === null)) {
        throw new Error("this shouldn't happen");
    }

    file.value.CstmrCdtTrfInitn.PmtInf.CdtTrfTxInf[transactionToEdit.value].Cdtr.Nm = name
    file.value.CstmrCdtTrfInitn.PmtInf.CdtTrfTxInf[transactionToEdit.value].CdtrAcct.Id.IBAN = iban
    file.value.CstmrCdtTrfInitn.PmtInf.CdtTrfTxInf[transactionToEdit.value].CdtrAgt.FinInstnId.BIC = bic
    file.value.CstmrCdtTrfInitn.PmtInf.CdtTrfTxInf[transactionToEdit.value].Amt.InstdAmt.$value = amount

    recomputeHeaders();

    editModalVisible.value = false;
    transactionToEdit.value = null;

    toast.add({ severity: "success", summary: 'Alterações efetuadas com sucesso!', life: 5000 });
}

function addTransaction(event: FormSubmitEvent) {
    if (!event.valid) {
        toast.add({ severity: "error", summary: 'Foram detetados erros na submissão. Por favor corrija e tente novamente.', life: 5000 });
        return;
    }

    const name = event.states.name.value
    const iban = event.states.iban.value
    const bic = event.states.bic.value
    const amount = event.states.amount.value

    if (file.value === null) {
        throw new Error("this shouldn't happen");
    }

    const transaction: CreditorTransferTransactionInformation = {
        PmtId: { EndToEndId: "NOTPROVIDED" },
        Cdtr: { Nm: name },
        CdtrAcct: { Id: { IBAN: iban } },
        CdtrAgt: { FinInstnId: { BIC: bic } },
        Amt: { InstdAmt: { "@Ccy": "EUR", $value: amount } },
    }

    file.value.CstmrCdtTrfInitn.PmtInf.CdtTrfTxInf.push(transaction);

    recomputeHeaders();

    addModalVisible.value = false;

    toast.add({ severity: "success", summary: 'Transação adicionada com sucesso!', life: 5000 });
}

function editHeader(event: FormSubmitEvent) {
    if (!event.valid) {
        toast.add({ severity: "error", summary: 'Foram detetados erros na submissão. Por favor corrija e tente novamente.', life: 5000 });
        return;
    }

    const id = event.states.id.value
    const creation_date: string = event.states.creation_date.value
    const initiating_party_name = event.states.initiating_party_name.value

    if (file.value === null) {
        throw new Error("this shouldn't happen");
    }

    file.value.CstmrCdtTrfInitn.GrpHdr.MsgId = id;
    file.value.CstmrCdtTrfInitn.GrpHdr.CreDtTm = creation_date;
    file.value.CstmrCdtTrfInitn.GrpHdr.InitgPty.Nm = initiating_party_name;

    file.value.CstmrCdtTrfInitn.PmtInf.PmtInfId = id;
    file.value.CstmrCdtTrfInitn.PmtInf.ReqdExctnDt = creation_date.slice(0, 10);
    file.value.CstmrCdtTrfInitn.PmtInf.Dbtr.Nm = initiating_party_name;

    recomputeHeaders();

    editHeaderModalVisible.value = false;

    toast.add({ severity: "success", summary: 'Alterações efetuadas com sucesso!', life: 5000 });
}

function removeTransaction(i: number): void {
    if (file.value === null) {
        throw new Error("this shouldn't happen");
    }

    file.value.CstmrCdtTrfInitn.PmtInf.CdtTrfTxInf.splice(i, 1);

    recomputeHeaders();

    toast.add({ severity: "success", summary: 'Transacção removida com sucesso.', life: 2000 });
}

function recomputeHeaders() {
    if (file.value === null) {
        throw new Error("this shouldn't happen");
    }

    const numberOfTransactions = file.value.CstmrCdtTrfInitn.PmtInf.CdtTrfTxInf.length;

    file.value.CstmrCdtTrfInitn.GrpHdr.NbOfTxs = numberOfTransactions;
    file.value.CstmrCdtTrfInitn.PmtInf.NbOfTxs = numberOfTransactions;

    const controlSum = file.value.CstmrCdtTrfInitn.PmtInf.CdtTrfTxInf.reduce((acc, transaction) => {
        return acc.add(transaction.Amt.InstdAmt.$value)
    }, currency(0, { separator: "", decimal: ".", symbol: "€", precision: 2 })).toString();

    file.value.CstmrCdtTrfInitn.GrpHdr.CtrlSum = controlSum;
    file.value.CstmrCdtTrfInitn.PmtInf.CtrlSum = controlSum;
}

const headerResolver = zodResolver(z.object(
    {
        initiating_party_name: z.string({ required_error: "Por favor introduza o nome." })
            .min(1, "Por favor introduza o nome."),
        id: z.string({ required_error: "Por favor introduza a identificação." })
            .min(1, "Por favor introduza a identificação."),
        creation_date: z.string({ required_error: "Por favor introduza a data." })
            .datetime({ local: true, message: "Data inválida. Deve seguir o formato YYYY-MM-DDTHH:mm:ss (e.g. 2024-12-22T12:00:12)." },)
    }
))

const transactionResolver = zodResolver(z.object(
    {
        name: z.string({ required_error: "Por favor introduza o nome." })
            .min(1, "Por favor introduza um nome."),
        iban: z.string({ required_error: "Por favor introduza o IBAN destino." })
            .refine((arg) => ibantools.isValidIBAN(arg), "IBAN inválido."),
        bic: z.string({ required_error: "Por favor introduza o BIC da conta destino." })
            .refine((arg) => ibantools.isValidBIC(arg), "BIC inválido."),
        amount: z.string({ required_error: "Por favor introduza a quantia." })
            .regex(/^\d+\.\d{2}$/, "Quantia inválida. A quantia deve ser indicada usando o ponto como separador decimal, contendo sempre duas casas decimais. Por exemplo, 1235.23 e 85.00 são considerados valores válidos."),
    }
))
</script>

<template>
    <main>
        <Toast />
        <div class="flex justify-between items-baseline">
            <h1 class="font-bold text-lg">Editor de Ficheiros SEPA XML</h1>

            <div v-if="file !== null" class="flex gap-4">
                <form class="row" @submit.prevent="saveFile">
                    <Button type="submit">Guardar</Button>
                </form>
                <form class="row" @submit.prevent="clearFile">
                    <Button type="submit">Limpar</Button>
                </form>
            </div>
            <FileUpload v-else mode="basic" name="demo[]" accept="text/xml" :maxFileSize="1000000" @uploader="onUpload"
                :auto="true" chooseLabel="Carregar ficheiro" custom-upload />
        </div>

        <div v-if="file !== null">
            <Card class="mb-4 mt-8">
                <template #content>
                    <h2 class="font-bold text-lg">Resumo</h2>
                    <div>Ordenante: {{ file?.CstmrCdtTrfInitn.GrpHdr.InitgPty.Nm }}</div>
                    <div>Identificação do pedido: {{ file?.CstmrCdtTrfInitn.GrpHdr.MsgId }}</div>
                    <div>Data de criação: {{ file?.CstmrCdtTrfInitn.GrpHdr ? new
                        Date(file?.CstmrCdtTrfInitn.GrpHdr.CreDtTm).toLocaleString() : null }}
                    </div>
                    <div>{{ file?.CstmrCdtTrfInitn.GrpHdr.NbOfTxs }} transações</div>
                    <div>{{ file?.CstmrCdtTrfInitn.GrpHdr.CtrlSum }}€ no total</div>

                    <Button class="mt-2" @click="editHeaderModalVisible = true">
                        <i class="pi pi-file-edit" style="font-size: 0.75rem"></i>Editar
                    </Button>
                </template>
            </Card>

            <Card v-for="(transaction, i) in file.CstmrCdtTrfInitn.PmtInf.CdtTrfTxInf"
                :key="transaction.CdtrAcct.Id.IBAN" class="my-4">
                <template #content>
                    <div class="flex justify-between text-lg">
                        <div>{{ transaction.Cdtr.Nm }}</div>
                        <div>{{ transaction.Amt.InstdAmt.$value }}€</div>
                    </div>
                    <div class="flex justify-between text-sm">
                        <div>{{ transaction.CdtrAcct.Id.IBAN }}</div>
                        <div>{{ transaction.CdtrAgt.FinInstnId.BIC }}</div>
                    </div>

                    <div class="flex gap-4 mt-2 justify-end">
                        <Button class="mt-2" @click="() => { editModalVisible = true; transactionToEdit = i }">
                            <i class="pi pi-file-edit" style="font-size: 0.75rem"></i>Editar
                        </Button>
                        <Button class="mt-2" @click="() => removeTransaction(i)">
                            <i class="pi pi-trash" style="font-size: 0.75rem"></i>Remover
                        </Button>
                    </div>
                </template>
            </Card>

            <Button class="mt-2" @click="addModalVisible = true">
                <i class="pi pi-plus" style="font-size: 0.75rem"></i>Adicionar
            </Button>

            <Dialog v-model:visible="addModalVisible" modal header="Adicionar transacção" :style="{ width: '30rem' }">
                <Form v-slot="$form" @submit="addTransaction" :resolver="transactionResolver">
                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="name" class="font-semibold w-24">Nome</label>
                            <InputText id="name" name="name" class="flex-auto" autocomplete="off" />
                        </div>
                        <Message v-if="$form.name?.invalid" severity="error" size="small" variant="simple">{{
                            $form.name.error?.message }}</Message>
                    </div>

                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="iban" class="font-semibold w-24">IBAN</label>
                            <InputText id="iban" name="iban" class="flex-auto" autocomplete="off" />
                        </div>
                        <Message v-if="$form.iban?.invalid" severity="error" size="small" variant="simple">{{
                            $form.iban.error?.message }}</Message>
                    </div>

                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="bic" class="font-semibold w-24">BIC</label>
                            <InputText id="bic" name="bic" class="flex-auto" autocomplete="off" />
                        </div>
                        <Message v-if="$form.bic?.invalid" severity="error" size="small" variant="simple">{{
                            $form.bic.error?.message }}</Message>
                    </div>

                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="amount" class="font-semibold w-24">Quantia</label>
                            <InputText id="amount" name="amount" class="flex-auto" autocomplete="off" />€
                        </div>
                        <Message v-if="$form.amount?.invalid" severity="error" size="small" variant="simple">{{
                            $form.amount.error?.message }}</Message>
                    </div>
                    <Button type="submit" severity="success" label="Adicionar" />
                </Form>
            </Dialog>

            <Dialog v-model:visible="editModalVisible" modal header="Editar transacção" :style="{ width: '30rem' }">
                <Form v-slot="$form" :initialValues @submit="editTransaction" :resolver="transactionResolver">
                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="name" class="font-semibold w-24">Nome</label>
                            <InputText id="name" name="name" class="flex-auto" autocomplete="off" />
                        </div>
                        <Message v-if="$form.name?.invalid" severity="error" size="small" variant="simple">{{
                            $form.name.error?.message }}</Message>
                    </div>

                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="iban" class="font-semibold w-24">IBAN</label>
                            <InputText id="iban" name="iban" class="flex-auto" autocomplete="off" />
                        </div>
                        <Message v-if="$form.iban?.invalid" severity="error" size="small" variant="simple">{{
                            $form.iban.error?.message }}</Message>
                    </div>

                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="bic" class="font-semibold w-24">BIC</label>
                            <InputText id="bic" name="bic" class="flex-auto" autocomplete="off" />
                        </div>
                        <Message v-if="$form.bic?.invalid" severity="error" size="small" variant="simple">{{
                            $form.bic.error?.message }}</Message>
                    </div>

                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="amount" class="font-semibold w-24">Quantia</label>
                            <InputText id="amount" name="amount" class="flex-auto" autocomplete="off" />€
                        </div>
                        <Message v-if="$form.amount?.invalid" severity="error" size="small" variant="simple">{{
                            $form.amount.error?.message }}</Message>
                    </div>
                    <Button type="submit" severity="secondary" label="Guardar" />
                </Form>
            </Dialog>

            <Dialog v-model:visible="editHeaderModalVisible" modal header="Editar cabeçalho"
                :style="{ width: '30rem' }">
                <Form v-slot="$form" :initial-values="initialHeaderValues" :resolver="headerResolver"
                    @submit="editHeader">
                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="initiating_party_name" class="font-semibold w-24">Ordenante</label>
                            <InputText id="initiating_party_name" name="initiating_party_name" class="flex-auto"
                                autocomplete="off" />
                        </div>
                        <Message v-if="$form.initiating_party_name?.invalid" severity="error" size="small"
                            variant="simple">
                            {{ $form.initiating_party_name.error?.message }}
                        </Message>
                    </div>

                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="id" class="font-semibold w-24">Identificação do pedido</label>
                            <InputText id="id" name="id" class="flex-auto" autocomplete="off" />
                        </div>
                        <Message v-if="$form.id?.invalid" severity="error" size="small" variant="simple">
                            {{ $form.id.error?.message }}
                        </Message>
                    </div>

                    <div class="mb-4">
                        <div class="flex items-center gap-4">
                            <label for="creation_date" class="font-semibold w-24">Data de criação</label>
                            <InputText id="creation_date" name="creation_date" class="flex-auto" autocomplete="off" />
                        </div>
                        <Message v-if="$form.creation_date?.invalid" severity="error" size="small" variant="simple">
                            {{ $form.creation_date.error?.message }}
                        </Message>
                    </div>
                    <Button type="submit" severity="secondary" label="Guardar" />
                </Form>
            </Dialog>
        </div>
    </main>
</template>
