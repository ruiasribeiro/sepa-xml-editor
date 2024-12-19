<script setup lang="ts">
import { computed, getCurrentInstance, type Ref, ref } from "vue";

import Button from 'primevue/button';
import Card from 'primevue/card';
import Dialog from 'primevue/dialog';
import FileUpload, { type FileUploadUploaderEvent } from 'primevue/fileupload';
import InputText from 'primevue/inputtext';

import { Form, type FormSubmitEvent } from '@primevue/forms';

import 'primeicons/primeicons.css'

import init, { Document, edit_headers, add_transaction, edit_transaction, parse_document, recompute_headers, remove_transaction, write_xml } from "../external/sepa-xml-editor-core/sepa_xml_editor_core";

init()

const file: Ref<Document | null, Document | null> = ref(null);
const transactionToEdit: Ref<number | null, number | null> = ref(null);
const addModalVisible = ref(false);
const editModalVisible = ref(false);
const editHeaderModalVisible = ref(false);

const header: Ref<{ id: string, creation_date: string, number_of_transactions: number, control_sum: string, initiating_party_name: string } | undefined, { id: string, creation_date: string, number_of_transactions: number, control_sum: string, initiating_party_name: string } | undefined> = ref(undefined);

function clearFile() {
    file.value = null;
}

function saveFile() {
    if (file.value === null) {
        throw new Error("");
    }

    // file.value.customer_credit_tranfer_initiation.header.id = "so-para-testar";

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
    recomputeHeaders()
}

const initialValues = computed(() => {
    if (transactionToEdit.value === null) {
        return {};
    }

    const transaction = file.value?.customer_credit_transfer_initiation.payment_information.transactions[transactionToEdit.value];

    return {
        "name": transaction?.creditor.name,
        "iban": transaction?.creditor_account.id.iban,
        "bic": transaction?.creditor_agent.financial_institution_id.bic,
        "amount": transaction?.amount.instructed_amount.value,
    };
});

const initialHeaderValues = computed(() => {
    if (file.value === null) {
        return {};
    }

    const header = file.value?.customer_credit_transfer_initiation.header;

    return {
        "id": header.id,
        "creation_date": header.creation_date,
        "initiating_party_name": header.initiating_party.name,
    };
});

const transactions = computed(() => {
    if (file.value === null) {
        return [];
    }

    return file.value.customer_credit_transfer_initiation.payment_information.transactions;
})

function onFormSubmit(event: FormSubmitEvent) {
    const name = event.states.name.value
    const iban = event.states.iban.value
    const bic = event.states.bic.value
    const amount = event.states.amount.value

    if ((file.value === null) || (transactionToEdit.value === null)) {
        throw new Error("this shouldn't happen");
    }

    edit_transaction(file.value, transactionToEdit.value, name, iban, bic, amount);
    recomputeHeaders();

    editModalVisible.value = false;
    transactionToEdit.value = null;
}

function addTransaction(event: FormSubmitEvent) {
    const name = event.states.name.value
    const iban = event.states.iban.value
    const bic = event.states.bic.value
    const amount = event.states.amount.value

    if (file.value === null) {
        throw new Error("this shouldn't happen");
    }

    add_transaction(file.value, name, iban, bic, amount);
    recomputeHeaders();

    addModalVisible.value = false;
}

function editHeader(event: FormSubmitEvent) {
    const id = event.states.id.value
    const creation_date = event.states.creation_date.value
    const initiating_party_name = event.states.initiating_party_name.value

    if (file.value === null) {
        throw new Error("this shouldn't happen");
    }

    edit_headers(file.value, id, creation_date, initiating_party_name);
    recomputeHeaders();

    editHeaderModalVisible.value = false;
}

function removeTransaction(i: number): void {
    if (file.value === null) {
        throw new Error("this shouldn't happen");
    }

    remove_transaction(file.value, i);
    recomputeHeaders();
    // const instance = getCurrentInstance();
    // instance?.proxy?.$forceUpdate();

    // this is an hack to force refresh of the list
    editModalVisible.value = true;
    editModalVisible.value = false;
    console.log("removed")
}

function recomputeHeaders() {
    if (file.value === null) {
        throw new Error("this shouldn't happen");
    }

    recompute_headers(file.value);

    header.value = { id: file.value.customer_credit_transfer_initiation.header.id, creation_date: file.value.customer_credit_transfer_initiation.header.creation_date, number_of_transactions: file.value.customer_credit_transfer_initiation.header.number_of_transactions, control_sum: file.value.customer_credit_transfer_initiation.header.control_sum, initiating_party_name: file.value.customer_credit_transfer_initiation.header.initiating_party.name }
}
</script>

<template>
    <main>
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
            <div v-if="header !== null">
                <Card class="mb-4 mt-8">
                    <template #content>
                        <h2 class="font-bold text-lg">Resumo</h2>
                        <div>Ordenante: {{ header?.initiating_party_name }}</div>
                        <div>Identificação do pedido: {{ header?.id }}</div>
                        <div>Data de criação: {{ header ? new Date(header?.creation_date).toLocaleString() : null }}
                        </div>
                        <div>{{ header?.number_of_transactions }} transações</div>
                        <div>{{ header?.control_sum }}€ no total</div>

                        <Button class="mt-2" @click="editHeaderModalVisible = true">
                            <i class="pi pi-file-edit" style="font-size: 0.75rem"></i>Editar
                        </Button>
                    </template>
                </Card>
            </div>

            <Card v-for="(transaction, i) in file.customer_credit_transfer_initiation.payment_information.transactions"
                :key="transaction.creditor_account.id.iban" class="my-4">
                <template #content>
                    <div class="flex justify-between text-lg">
                        <div>{{ transaction.creditor.name }}</div>
                        <div>{{ transaction.amount.instructed_amount.value }}€</div>
                    </div>
                    <div class="flex justify-between text-sm">
                        <div>{{ transaction.creditor_account.id.iban }}</div>
                        <div>{{ transaction.creditor_agent.financial_institution_id.bic }}</div>
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
                <Form v-slot="$form" @submit="addTransaction">
                    <div class="flex items-center gap-4 mb-4">
                        <label for="name" class="font-semibold w-24">Nome</label>
                        <InputText id="name" name="name" class="flex-auto" autocomplete="off" />
                    </div>
                    <div class="flex items-center gap-4 mb-4">
                        <label for="iban" class="font-semibold w-24">IBAN</label>
                        <InputText id="iban" name="iban" class="flex-auto" autocomplete="off" />
                    </div>
                    <div class="flex items-center gap-4 mb-4">
                        <label for="bic" class="font-semibold w-24">BIC</label>
                        <InputText id="bic" name="bic" class="flex-auto" autocomplete="off" />
                    </div>
                    <div class="flex items-center gap-4 mb-8">
                        <label for="amount" class="font-semibold w-24">Quantia</label>
                        <InputText id="amount" name="amount" class="flex-auto" autocomplete="off" />€
                    </div>
                    <Button type="submit" severity="secondary" label="Adicionar" />
                </Form>
            </Dialog>

            <Dialog v-model:visible="editModalVisible" modal header="Editar transacção" :style="{ width: '30rem' }">
                <Form v-slot="$form" :initialValues @submit="onFormSubmit">
                    <div class="flex items-center gap-4 mb-4">
                        <label for="name" class="font-semibold w-24">Nome</label>
                        <InputText id="name" name="name" class="flex-auto" autocomplete="off" />
                    </div>
                    <div class="flex items-center gap-4 mb-4">
                        <label for="iban" class="font-semibold w-24">IBAN</label>
                        <InputText id="iban" name="iban" class="flex-auto" autocomplete="off" />
                    </div>
                    <div class="flex items-center gap-4 mb-4">
                        <label for="bic" class="font-semibold w-24">BIC</label>
                        <InputText id="bic" name="bic" class="flex-auto" autocomplete="off" />
                    </div>
                    <div class="flex items-center gap-4 mb-8">
                        <label for="amount" class="font-semibold w-24">Quantia</label>
                        <InputText id="amount" name="amount" class="flex-auto" autocomplete="off" />€
                    </div>
                    <Button type="submit" severity="secondary" label="Guardar" />
                </Form>
            </Dialog>

            <Dialog v-model:visible="editHeaderModalVisible" modal header="Editar cabeçalho"
                :style="{ width: '30rem' }">
                <Form v-slot="$form" :initial-values="header" @submit="editHeader">
                    <div class="flex items-center gap-4 mb-4">
                        <label for="initiating_party_name" class="font-semibold w-24">Ordenante</label>
                        <InputText id="initiating_party_name" name="initiating_party_name" class="flex-auto"
                            autocomplete="off" />
                    </div>
                    <div class="flex items-center gap-4 mb-4">
                        <label for="id" class="font-semibold w-24">Identificação do pedido</label>
                        <InputText id="id" name="id" class="flex-auto" autocomplete="off" />
                    </div>
                    <div class="flex items-center gap-4 mb-4">
                        <label for="creation_date" class="font-semibold w-24">Data de criação</label>
                        <InputText id="creation_date" name="creation_date" class="flex-auto" autocomplete="off" />
                    </div>
                    <Button type="submit" severity="secondary" label="Guardar" />
                </Form>
            </Dialog>
        </div>

    </main>
</template>
