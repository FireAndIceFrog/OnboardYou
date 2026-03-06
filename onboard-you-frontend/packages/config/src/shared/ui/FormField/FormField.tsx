import { Field, Input } from "@chakra-ui/react"
import { FieldError } from "../FieldError/FieldError"
import { ChangeEvent } from "react"

export interface FormFieldProps {
    error?: string,
    helperText?: string,
    id?: string,
    label: string,
    min?: number,
    max?: number,
    onBlur?: () => void,
    placeholder?: string,
    step?: number,
    value: string | number,
    onChange: (value: ChangeEvent<HTMLInputElement, HTMLInputElement>) => void,
    type?: string
}
export const FormField = (props: FormFieldProps) => {
    let id = props.id || "form_field_" + Math.random().toString(36).substring(2, 15);
    return (
        <Field.Root invalid={!!props.error}>
            <Field.Label htmlFor={id}>{props.label}</Field.Label>
            <Input
            id={id}
            type={props.type || "text"}
            placeholder={props.placeholder}
            value={props.value}
            min={props.min}
            max={props.max}
            step={props.step}
            onBlur={props.onBlur}
            onChange={(e) => props.onChange(e)}
            />
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