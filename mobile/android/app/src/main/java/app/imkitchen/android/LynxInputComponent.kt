// Copyright 2025 The Lynx Authors. All rights reserved.
// Licensed under the Apache License Version 2.0 that can be found in the
// LICENSE file in the root directory of this source tree.
package app.imkitchen.android

import android.content.Context
import android.graphics.Color
import android.text.Editable
import android.text.InputFilter
import android.text.InputType
import android.text.TextWatcher
import android.text.method.PasswordTransformationMethod
import android.text.method.SingleLineTransformationMethod
import android.view.Gravity
import android.view.View
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager
import androidx.appcompat.widget.AppCompatEditText
import com.lynx.react.bridge.Callback
import com.lynx.react.bridge.ReadableMap
import com.lynx.tasm.behavior.LynxContext
import com.lynx.tasm.behavior.LynxProp
import com.lynx.tasm.behavior.LynxUIMethod
import com.lynx.tasm.behavior.LynxUIMethodConstants
import com.lynx.tasm.behavior.ui.LynxUI
import com.lynx.tasm.event.LynxCustomEvent


class LynxInputComponent(context: LynxContext?) : LynxUI<AppCompatEditText>(context) {

  private var multiline = false
  private var password = false

  override fun createView(context: Context): AppCompatEditText {
    return AppCompatEditText(context).apply {
      setLines(1)
      setSingleLine()
      gravity = Gravity.CENTER_VERTICAL
      background = null
      imeOptions = EditorInfo.IME_ACTION_NONE
      setHorizontallyScrolling(true)
      setPadding(0, 0, 0, 0)
      addTextChangedListener(object : TextWatcher {
        override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {}
        override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {}
        override fun afterTextChanged(s: Editable?) {
          emitEvent("input", mapOf("value" to (s?.toString() ?: "")))
        }
      })
      onFocusChangeListener = View.OnFocusChangeListener { _, hasFocus ->
        emitEvent(if (hasFocus) "focus" else "blur", null)
      }
      setOnEditorActionListener { _, actionId, _ ->
        if (actionId == EditorInfo.IME_ACTION_NONE) {
          false
        } else {
          emitEvent("confirm", mapOf("value" to text.toString()))
          true
        }
      }
    }
  }

  /** `text` (default) | `password` | `email` | `number` | `digit` | `tel`. */
  @LynxProp(name = "type")
  fun setType(value: String) {
    password = value == "password"
    mView.inputType = when (value) {
      "password" -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
      "email" -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS
      "number" -> InputType.TYPE_CLASS_NUMBER
      "digit" -> InputType.TYPE_CLASS_NUMBER or InputType.TYPE_NUMBER_FLAG_DECIMAL
      "tel" -> InputType.TYPE_CLASS_PHONE
      else -> InputType.TYPE_CLASS_TEXT
    }
    applyLineMode()
  }

  /**
   * `true` turns the field into a growing text area: wrapped lines, the
   * keyboard's return key inserts a newline, text starts at the top.
   */
  @LynxProp(name = "multiline")
  fun setMultiline(value: Boolean) {
    multiline = value
    applyLineMode()
  }

  /**
   * setInputType resets the line mode, and setSingleLine installs its own
   * transformation method (which would undo password masking), so both props
   * re-apply the whole set here.
   */
  private fun applyLineMode() {
    if (multiline && !password) {
      mView.inputType = mView.inputType or InputType.TYPE_TEXT_FLAG_MULTI_LINE
      mView.setSingleLine(false)
      mView.maxLines = Int.MAX_VALUE
      mView.setHorizontallyScrolling(false)
      mView.gravity = Gravity.TOP or Gravity.START
      mView.transformationMethod = null
    } else {
      mView.setSingleLine()
      mView.setHorizontallyScrolling(true)
      mView.gravity = Gravity.CENTER_VERTICAL
      mView.transformationMethod = if (password) {
        PasswordTransformationMethod.getInstance()
      } else {
        SingleLineTransformationMethod.getInstance()
      }
    }
    mView.typeface = android.graphics.Typeface.DEFAULT
    mView.setSelection(mView.text?.length ?: 0)
  }

  @LynxProp(name = "maxlength")
  fun setMaxLength(value: Int) {
    mView.filters = if (value > 0) arrayOf(InputFilter.LengthFilter(value)) else emptyArray()
  }

  /** Keyboard action button: `done` | `next` | `go` | `search` | `send`. */
  @LynxProp(name = "confirm-type")
  fun setConfirmType(value: String?) {
    mView.imeOptions = when (value) {
      "done" -> EditorInfo.IME_ACTION_DONE
      "next" -> EditorInfo.IME_ACTION_NEXT
      "go" -> EditorInfo.IME_ACTION_GO
      "search" -> EditorInfo.IME_ACTION_SEARCH
      "send" -> EditorInfo.IME_ACTION_SEND
      else -> EditorInfo.IME_ACTION_NONE
    }
  }

  @LynxUIMethod
  fun blur(params: ReadableMap, callback: Callback) {
    mView.clearFocus()
    val imm = lynxContext.getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
    imm.hideSoftInputFromWindow(mView.windowToken, 0)
    callback.invoke(LynxUIMethodConstants.SUCCESS)
  }

  override fun onLayoutUpdated() {
    super.onLayoutUpdated()
    val paddingTop = mPaddingTop + mBorderTopWidth
    val paddingBottom = mPaddingBottom + mBorderBottomWidth
    val paddingLeft = mPaddingLeft + mBorderLeftWidth
    val paddingRight = mPaddingRight + mBorderRightWidth
    mView.setPadding(paddingLeft, paddingTop, paddingRight, paddingBottom)
  }

  @LynxProp(name = "value")
  fun setValue(value: String) {
    if (value != mView.text.toString()) {
      mView.setText(value)
      // setText parks the cursor at 0; keep typing at the end.
      mView.setSelection(mView.text?.length ?: 0)
    }
  }

  @LynxUIMethod
  fun focus(params: ReadableMap, callback: Callback) {
    if (mView.requestFocus()) {
      if (showSoftInput()) {
        callback.invoke(LynxUIMethodConstants.SUCCESS)
      } else {
        callback.invoke(LynxUIMethodConstants.UNKNOWN, "fail to show keyboard")
      }
    } else {
      callback.invoke(LynxUIMethodConstants.UNKNOWN, "fail to focus")
    }
  }

  private fun showSoftInput(): Boolean {
    val imm = lynxContext.getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
    return imm.showSoftInput(mView, InputMethodManager.SHOW_IMPLICIT, null)
  }

  @LynxProp(name = "placeholder")
  fun setPlaceHolder(value: String?) {
    mView.hint = value ?: ""
  }

  @LynxProp(name = "text-color")
  fun setTextColor(value: String) {
    var value = value
    if (value.startsWith("#")) {
      value = value.substring(1)
    }
    val textColor = "#" + value
    val hintColor = "#40" + value
    mView.setHintTextColor(Color.parseColor(hintColor))
    mView.setTextColor(Color.parseColor(textColor))
  }



  private fun emitEvent(name: String, value: Map<String, Any>?) {
    val detail = LynxCustomEvent(sign, name)
    value?.forEach { (key, v) -> detail.addDetail(key, v) }
    lynxContext.eventEmitter.sendCustomEvent(detail)
  }
}
