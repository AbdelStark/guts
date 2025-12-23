/**
 * Label types.
 */

export interface Label {
  /** Label name */
  name: string;
  /** Label color (hex without #) */
  color: string;
  /** Label description */
  description?: string;
}

export interface CreateLabelRequest {
  /** Label name */
  name: string;
  /** Label color (hex without #) */
  color: string;
  /** Label description */
  description?: string;
}

export interface UpdateLabelRequest {
  /** New label name */
  new_name?: string;
  /** Label color (hex without #) */
  color?: string;
  /** Label description */
  description?: string;
}
