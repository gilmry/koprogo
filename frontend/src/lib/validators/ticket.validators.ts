import type { CreateTicketDto } from "../api/tickets";
import {
  required,
  minLength,
  collectErrors,
  type ValidationResult,
} from "./common.validators";

/**
 * Validate a ticket creation form.
 * Returns a Record of field → error message (empty if valid).
 */
export function validateCreateTicket(
  data: CreateTicketDto,
  messages?: {
    buildingRequired?: string;
    titleMinLength?: string;
    descriptionMinLength?: string;
  },
): Record<string, string> {
  const errors: Record<string, ValidationResult> = {
    building_id: required(data.building_id, messages?.buildingRequired),
    title: minLength(data.title, 3, messages?.titleMinLength),
    description: minLength(
      data.description,
      10,
      messages?.descriptionMinLength,
    ),
  };
  return collectErrors(errors);
}
