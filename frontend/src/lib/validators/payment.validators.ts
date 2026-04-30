import type { CreatePaymentMethodDto } from "../api/payments";
import { PaymentMethodType } from "../api/payments";
import {
  minLength,
  startsWithPrefix,
  collectErrors,
  type ValidationResult,
} from "./common.validators";

/**
 * Validate a payment method creation form.
 * Returns a Record of field → error message (empty if valid).
 */
export function validatePaymentMethod(
  data: CreatePaymentMethodDto,
  messages?: {
    labelMinLength?: string;
    stripeIdRequired?: string;
  },
): Record<string, string> {
  const errors: Record<string, ValidationResult> = {
    display_label: minLength(data.display_label, 3, messages?.labelMinLength),
  };

  // Stripe payment method ID required for Card and SepaDebit
  if (
    data.method_type === PaymentMethodType.Card ||
    data.method_type === PaymentMethodType.SepaDebit
  ) {
    errors.stripe_payment_method_id = startsWithPrefix(
      data.stripe_payment_method_id,
      "pm_",
      messages?.stripeIdRequired ??
        "Valid Stripe Payment Method ID required (starts with pm_)",
    );
  }

  return collectErrors(errors);
}
