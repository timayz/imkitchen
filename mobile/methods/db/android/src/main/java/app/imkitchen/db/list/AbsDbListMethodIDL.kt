package app.imkitchen.db.list

import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodName
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamField
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamModel
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodResultModel
import com.tiktok.sparkling.method.registry.core.base.AbsSparklingIDLMethod
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseParamModel
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseResultModel

/** IDL of `Db.list`; see `DbStore` for the semantics. */
abstract class AbsDbListMethodIDL :
    AbsSparklingIDLMethod<AbsDbListMethodIDL.Params, AbsDbListMethodIDL.Result>() {

    @IDLMethodName(name = "Db.list", params = ["collection", "prefix"], results = ["json"])
    final override val name: String = "Db.list"

    @IDLMethodParamModel
    interface Params : IDLMethodBaseParamModel {

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "collection")
        val collection: String

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "prefix")
        val prefix: String?
    }

    @IDLMethodResultModel
    interface Result : IDLMethodBaseResultModel {

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "json")
        @set:IDLMethodParamField(required = false, isGetter = false, keyPath = "json")
        var json: String?
    }
}
