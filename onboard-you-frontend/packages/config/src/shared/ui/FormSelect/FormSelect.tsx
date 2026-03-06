import { Field, NativeSelect } from "@chakra-ui/react"
import { FieldError } from "../FieldError/FieldError"
import { ChangeEvent } from "react"

export interface FormSelectProps {
    error?: string,
    helperText?: string,
    id?: string,
    label: string,
    placeholder?: string,
    value: string,
    onChange: (value: ChangeEvent<HTMLSelectElement, HTMLSelectElement>) => void
    children: React.ReactNode
}

export const FormSelect = (props: FormSelectProps) => {
    let id = props.id || "form_field_" + Math.random().toString(36).substring(2, 15);
    return (
        <Field.Root invalid={!!props.error}>
            <Field.Label htmlFor={id}>{props.label}</Field.Label>
            <NativeSelect.Root>
                <NativeSelect.Field
                    id={id}
                    value={props.value}
                    onChange={props.onChange}
                >
                    {props.children}
                </NativeSelect.Field>
                <NativeSelect.Indicator />
            </NativeSelect.Root>
            {props.helperText && (
                <Field.HelperText>{props.helperText}</Field.HelperText>
            )}
            <FieldError
                id={id + "_error"}
                error={props.error}
            />
        </Field.Root>
    )

}